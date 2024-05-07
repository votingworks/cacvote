//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing::Level;
use types_rs::cacvote::{self, JournalEntry, JurisdictionCode, SignedObject};
use uuid::Uuid;

use crate::{
    bulletin_board,
    config::{Config, MAX_REQUEST_SIZE},
    db,
};

/// Prepares the application to be run within an HTTP server.
///
/// Requires a [`PgPool`] from [`db::setup`]. Run the application with [`run`]
/// with the result of this function.
pub async fn setup(pool: PgPool) -> color_eyre::Result<Router> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up application").entered();
    Ok(Router::new()
        .route("/api/status", get(get_status))
        .route("/api/objects", post(create_object))
        .route("/api/objects/:object_id", get(get_object_by_id))
        .route("/api/journal-entries", get(get_journal_entries))
        .route("/api/elections", get(list_elections))
        .route(
            "/api/elections/:election_id/cast-ballots",
            get(list_cast_ballots_by_election),
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
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state(pool))
}

/// Create and run an HTTP server using the provided application at the port
/// from [`config`][`super::config`].
pub async fn run(app: Router, config: &Config) -> color_eyre::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
    tracing::info!("Server listening at http://{addr}/");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

/// Always responds with a successful status. Used to check whether the server
/// is running.
async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

async fn create_object(
    State(pool): State<PgPool>,
    object: Json<SignedObject>,
) -> Result<impl IntoResponse, Error> {
    let mut conn = pool.acquire().await?;
    let object_id = db::create_object(&mut conn, &object).await?;
    Ok((StatusCode::CREATED, object_id.to_string()))
}

#[derive(Debug, Deserialize)]
struct GetJournalEntriesQuery {
    #[serde(rename = "since")]
    since_journal_entry_id: Option<Uuid>,

    #[serde(rename = "jurisdiction")]
    jurisdiction_code: Option<JurisdictionCode>,
}

async fn get_journal_entries(
    State(pool): State<PgPool>,
    Query(query): Query<GetJournalEntriesQuery>,
) -> Result<Json<Vec<JournalEntry>>, Error> {
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
    State(pool): State<PgPool>,
    Path(object_id): Path<Uuid>,
) -> Result<Json<SignedObject>, Error> {
    let mut conn = pool.acquire().await?;

    match db::get_object_by_id(&mut conn, object_id).await? {
        Some(object) => {
            tracing::info!("PAYLOAD: {}", std::str::from_utf8(&object.payload).unwrap());
            Ok(Json(object))
        }
        None => Err(Error::NotFound),
    }
}

async fn list_elections(
    State(pool): State<PgPool>,
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
    State(pool): State<PgPool>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<Vec<bulletin_board::CastBallot>>, Error> {
    let mut conn = pool.acquire().await?;

    Ok(Json(
        db::get_cast_ballot_ids_by_election(&mut conn, election_id)
            .await?
            .into_iter()
            .map(|id| bulletin_board::CastBallot::new(id, election_id.clone()))
            .collect(),
    ))
}

async fn get_cast_ballot_by_id(
    State(pool): State<PgPool>,
    Path((election_id, cast_ballot_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<SignedObject>, Error> {
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
    State(pool): State<PgPool>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<SignedObject>, Error> {
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
    State(pool): State<PgPool>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<SignedObject>, Error> {
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
    State(pool): State<PgPool>,
    Path(election_id): Path<Uuid>,
) -> Result<Json<SignedObject>, Error> {
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

#[derive(Debug, thiserror::Error)]
enum Error {
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
