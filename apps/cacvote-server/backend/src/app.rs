//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64_serde::base64_serde_type;
use cacvote_server_client::{
    CreateSessionRequest, CreateSessionRequestPayload, CreateSessionResponse,
};
use openssl::{hash::MessageDigest, sign::Verifier, x509};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::Level;
use types_rs::cacvote::{self, verify_cert_single_ca, ScannedMailingLabel};
use uuid::Uuid;

use crate::{
    bulletin_board,
    config::{Config, MAX_REQUEST_SIZE},
    db,
    session::{Session, SessionManager},
    state::AppState,
};

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

/// Prepares the application to be run within an HTTP server.
///
/// Requires a [`PgPool`] from [`db::setup`]. Run the application with [`run`]
/// with the result of this function.
pub async fn setup(
    pool: PgPool,
    vx_root_ca_cert: x509::X509,
    cac_root_ca_store: x509::store::X509Store,
) -> Router {
    let _entered = tracing::span!(Level::DEBUG, "Setting up application").entered();
    Router::new()
        .route("/api/status", get(get_status))
        .route("/api/sessions", post(create_session))
        .route("/api/objects", post(create_object))
        .route("/api/objects/:object_id", get(get_object_by_id))
        .route("/api/journal-entries", get(get_journal_entries))
        .route("/api/machines", post(create_machine))
        .route(
            "/api/scanned-mailing-label",
            post(scanned_create_mailing_label),
        )
        .route("/api/elections", get(list_elections))
        .route(
            "/api/elections/:election_id/cast-ballots",
            get(list_cast_ballots_by_election),
        )
        .route(
            "/api/elections/:election_id/scanned-mailing-labels",
            get(list_scanned_mailing_labels_by_election),
        )
        .route(
            "/api/elections/:election_id/cast-ballots/:cast_ballot_id",
            get(get_cast_ballot_by_id),
        )
        .route(
            "/api/elections/:election_id/encrypted-tally",
            get(get_encrypted_tally_by_election),
        )
        .route(
            "/api/elections/:election_id/decrypted-tally",
            get(get_decrypted_tally_by_election),
        )
        .route(
            "/api/elections/:election_id/shuffled-ballots",
            get(list_shuffled_ballots_by_election),
        )
        .route("/api/search", post(search))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            pool,
            vx_root_ca_cert,
            cac_root_ca_store: Arc::new(cac_root_ca_store),
            sessions: Arc::new(Mutex::new(SessionManager::new())),
        })
}

/// Create and run an HTTP server using the provided application at the port
/// from [`config`][`super::config`].
pub async fn run(app: Router, config: &Config) -> color_eyre::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
    tracing::info!("Server listening at http://{addr}/");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Always responds with a successful status. Used to check whether the server
/// is running.
async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

async fn create_session(
    State(AppState {
        vx_root_ca_cert,
        sessions,
        ..
    }): State<AppState>,
    Json(CreateSessionRequest {
        certificate,
        payload,
        signature,
    }): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, Error> {
    // verify "signed TPM public key" signed by config CA cert
    let certificate = x509::X509::from_pem(&certificate).map_err(|e| {
        tracing::error!("Failed to parse certificate: {e}");
        Error::BadRequest(format!("Failed to parse certificate: {e}"))
    })?;
    let public_key = certificate.public_key().map_err(|e| {
        tracing::error!("Failed to extract public key from certificate: {e}");
        Error::Other(e.into())
    })?;
    if !verify_cert_single_ca(&vx_root_ca_cert, &certificate).unwrap_or(false) {
        tracing::error!("Failed to verify certificate");
        return Err(Error::BadRequest("Failed to verify certificate".to_owned()));
    }

    // verify "signature" signed by "signed TPM public key"
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key).map_err(|e| {
        tracing::error!("Failed to create verifier: {e}");
        Error::BadRequest(format!("Failed to create verifier: {e}"))
    })?;
    verifier.update(payload.as_bytes()).map_err(|e| {
        tracing::error!("Failed to update verifier: {e}");
        Error::Other(e.into())
    })?;
    let verified = verifier.verify(&signature).map_err(|e| {
        tracing::error!("Failed to verify signature: {e}");
        Error::BadRequest(format!("Failed to verify signature: {e}"))
    })?;

    if !verified {
        tracing::error!("Signature verification failed");
        return Err(Error::BadRequest(
            "Signature verification failed".to_owned(),
        ));
    }

    // verify payload timestamp within N seconds of current time
    let payload: CreateSessionRequestPayload = serde_json::from_str(&payload).map_err(|e| {
        tracing::error!("Failed to parse payload: {e}");
        Error::BadRequest(format!("Failed to parse payload: {e}"))
    })?;

    let now = time::OffsetDateTime::now_utc();
    let max_age = time::Duration::seconds(30);
    if now - payload.timestamp > max_age {
        tracing::error!("Timestamp is too old");
        return Err(Error::BadRequest("Timestamp is too old".to_owned()));
    }

    // create a new authorization token
    let mut sessions = sessions.lock().await;
    let session = sessions.create(certificate).map_err(|e| {
        tracing::error!("Failed to create session: {e}");
        Error::BadRequest(format!("Failed to create session: {e}"))
    })?;
    let bearer_token = session.token().to_string();

    Ok((
        StatusCode::CREATED,
        Json(CreateSessionResponse { bearer_token }),
    ))
}

async fn create_object(
    _session: Session,
    State(AppState {
        vx_root_ca_cert,
        cac_root_ca_store,
        pool,
        ..
    }): State<AppState>,
    object: Json<cacvote::SignedObject>,
) -> Result<impl IntoResponse, Error> {
    match object.verify(&vx_root_ca_cert, &cac_root_ca_store) {
        Ok(true) => (),
        Ok(false) => {
            tracing::error!("Signature verification failed");
            return Err(Error::BadRequest(
                "Signature verification failed".to_owned(),
            ));
        }
        Err(e) => {
            tracing::error!("Error verifying object signature: {e}");
            return Err(Error::BadRequest(format!(
                "Error verifying object signature: {e}"
            )));
        }
    }
    let mut conn = pool.acquire().await?;
    let object_id = db::create_object(&mut conn, &object).await?;
    Ok((StatusCode::CREATED, object_id.to_string()))
}

#[derive(Debug, Deserialize)]
struct GetJournalEntriesQuery {
    #[serde(rename = "since")]
    since_journal_entry_id: Option<Uuid>,

    #[serde(rename = "jurisdiction")]
    jurisdiction_code: Option<cacvote::JurisdictionCode>,
}

async fn get_journal_entries(
    _session: Session,
    State(AppState { pool, .. }): State<AppState>,
    Query(query): Query<GetJournalEntriesQuery>,
) -> Result<Json<Vec<cacvote::JournalEntry>>, Error> {
    let mut conn = pool.acquire().await?;

    Ok(db::get_journal_entries(
        &mut conn,
        query.since_journal_entry_id,
        query.jurisdiction_code,
    )
    .await
    .map(Json)?)
}

async fn get_object_by_id(
    _session: Session,
    State(AppState { pool, .. }): State<AppState>,
    Path(object_id): Path<Uuid>,
) -> Result<Json<cacvote::SignedObject>, Error> {
    let mut conn = pool.acquire().await?;

    match db::get_object_by_id(&mut conn, object_id).await? {
        Some(object) => {
            tracing::info!("PAYLOAD: {}", std::str::from_utf8(&object.payload).unwrap());
            Ok(Json(object))
        }
        None => Err(Error::NotFound),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateMachineRequest {
    machine_identifier: String,

    #[serde(with = "Base64Standard")]
    certificate: Vec<u8>,
}

async fn create_machine(
    State(AppState {
        vx_root_ca_cert,
        pool,
        ..
    }): State<AppState>,
    Json(CreateMachineRequest {
        machine_identifier,
        certificate: certificate_bytes,
    }): Json<CreateMachineRequest>,
) -> Result<impl IntoResponse, Error> {
    let mut conn = pool.acquire().await?;

    if let Some(existing_machine) =
        db::get_machine_by_identifier(&mut conn, &machine_identifier).await?
    {
        if existing_machine.certificate == certificate_bytes {
            return Ok((StatusCode::OK, Json(json!({ "id": existing_machine.id }))));
        } else {
            tracing::error!("Machine already exists with different certificate");
            return Err(Error::BadRequest(
                "Machine already exists with different certificate".to_owned(),
            ));
        }
    }

    let certificate = openssl::x509::X509::from_pem(&certificate_bytes).map_err(|e| {
        tracing::error!("Failed to parse certificate: {e}");
        Error::BadRequest(format!("Failed to parse certificate: {e}"))
    })?;

    if !verify_cert_single_ca(&vx_root_ca_cert, &certificate).unwrap_or(false) {
        tracing::error!("Failed to verify certificate");
        return Err(Error::BadRequest("Failed to verify certificate".to_owned()));
    }

    // TODO: Extract the machine ID from the certificate rather than having it
    // be provided in the request. This prevents the client from lying about the
    // machine ID.
    let machine = db::create_machine(&mut conn, &machine_identifier, &certificate_bytes).await?;

    Ok((StatusCode::CREATED, Json(json!({ "id": machine.id }))))
}

async fn scanned_create_mailing_label(
    State(AppState { pool, .. }): State<AppState>,
    scanned_mailing_label_code: Bytes,
) -> Result<impl IntoResponse, Error> {
    let mut conn = pool.acquire().await?;

    let id = db::create_scanned_mailing_label_code(&mut conn, &scanned_mailing_label_code).await?;

    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn list_elections(
    State(AppState { pool, .. }): State<AppState>,
) -> Result<Json<Vec<bulletin_board::Election>>, Error> {
    let mut conn = pool.acquire().await?;

    Ok(Json(
        db::get_election_ids(&mut conn)
            .await?
            .into_iter()
            .map(bulletin_board::Election::new)
            .collect(),
    ))
}

async fn list_cast_ballots_by_election(
    State(AppState { pool, .. }): State<AppState>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<Vec<bulletin_board::CastBallot>>, Error> {
    let mut conn = pool.acquire().await?;

    Ok(Json(
        db::get_cast_ballot_ids_by_election(&mut conn, election_id)
            .await?
            .into_iter()
            .map(|id| bulletin_board::CastBallot::new(id, election_id))
            .collect(),
    ))
}

async fn list_scanned_mailing_labels_by_election(
    State(AppState { pool, .. }): State<AppState>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<Vec<ScannedMailingLabel>>, Error> {
    let mut conn = pool.acquire().await?;

    Ok(Json(
        db::get_scanned_mailing_label_codes(&mut conn, election_id).await?,
    ))
}

async fn get_cast_ballot_by_id(
    State(AppState { pool, .. }): State<AppState>,
    Path((election_id, cast_ballot_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<cacvote::SignedObject>, Error> {
    let mut conn = pool.acquire().await?;

    match db::get_object_by_id(&mut conn, cast_ballot_id).await? {
        Some(cast_ballot) => match cast_ballot.try_to_inner()? {
            cacvote::Payload::CastBallot(payload) if payload.election_object_id == election_id => {
                Ok(Json(cast_ballot))
            }
            _ => Err(Error::NotFound),
        },
        None => Err(Error::NotFound),
    }
}

async fn get_encrypted_tally_by_election(
    State(AppState { pool, .. }): State<AppState>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<cacvote::SignedObject>, Error> {
    let mut conn = pool.acquire().await?;

    match db::get_object_by_election_id_and_type(
        &mut conn,
        election_id,
        cacvote::Payload::encrypted_election_tally_object_type(),
    )
    .await?
    {
        Some(object) => Ok(Json(object)),
        None => Err(Error::NotFound),
    }
}

async fn get_decrypted_tally_by_election(
    State(AppState { pool, .. }): State<AppState>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<cacvote::SignedObject>, Error> {
    let mut conn = pool.acquire().await?;

    match db::get_object_by_election_id_and_type(
        &mut conn,
        election_id,
        cacvote::Payload::decrypted_election_tally_object_type(),
    )
    .await?
    {
        Some(object) => Ok(Json(object)),
        None => Err(Error::NotFound),
    }
}

async fn list_shuffled_ballots_by_election(
    State(AppState { pool, .. }): State<AppState>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<cacvote::SignedObject>, Error> {
    let mut conn = pool.acquire().await?;

    match db::get_object_by_election_id_and_type(
        &mut conn,
        election_id,
        cacvote::Payload::shuffled_encrypted_cast_ballots_object_type(),
    )
    .await?
    {
        Some(object) => Ok(Json(object)),
        None => Err(Error::NotFound),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchQuery {
    common_access_card_id: String,
}

async fn search(
    State(AppState { pool, .. }): State<AppState>,
    query: Query<SearchQuery>,
) -> Result<Json<Vec<db::SearchResult>>, Error> {
    let mut conn = pool.acquire().await?;

    Ok(Json(
        db::search(&mut conn, &query.common_access_card_id).await?,
    ))
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Not found")]
    NotFound,

    #[error("{0}")]
    Other(#[from] color_eyre::Report),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, json) = match self {
            Error::BadRequest(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            ),
            Error::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
            Error::Serde(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            ),
            error @ Error::NotFound => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": error.to_string() })),
            ),
            Error::Other(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
        };
        tracing::error!("Responding with error: {status} {json:?}");
        (status, json).into_response()
    }
}
