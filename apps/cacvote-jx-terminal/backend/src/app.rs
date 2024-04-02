//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use auth_rs::card_details::CardDetailsWithAuthInfo;
use axum::body::Bytes;
use axum::response::sse::{Event, KeepAlive};
use axum::response::Sse;
use axum::routing::post;
use axum::Json;
use axum::{extract::DefaultBodyLimit, routing::get, Router};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use futures::stream::Stream;
use serde_json::json;
use sqlx::PgPool;
use tokio_stream::StreamExt;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;
use types_rs::cacvote::{
    CreateRegistrationData, Election, Payload, Registration, SessionData, SignedObject,
};
use types_rs::election::ElectionDefinition;
use uuid::Uuid;

use crate::config::{Config, MAX_REQUEST_SIZE};
use crate::{db, smartcard};
use tokio::sync::broadcast;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    smartcard: smartcard::DynSmartcard,
    broadcast_tx: broadcast::Sender<SessionData>,
}

/// Prepares the application with all the routes. Run the application with
/// `app::run(…)` once you have it.
pub(crate) fn setup(pool: PgPool, config: Config, smartcard: smartcard::DynSmartcard) -> Router {
    let _entered = tracing::span!(Level::DEBUG, "Setting up application").entered();

    let router = match &config.public_dir {
        Some(public_dir) => Router::new().fallback_service(
            ServeDir::new(public_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(public_dir.join("index.html"))),
        ),
        None => {
            tracing::info!("No PUBLIC_DIR configured, serving no files");
            Router::new()
        }
    };

    let (broadcast_tx, _) = broadcast::channel(1);

    tokio::spawn({
        let jurisdiction_code = config.jurisdiction_code.clone();
        let pool = pool.clone();
        let smartcard = smartcard.clone();
        let broadcast_tx = broadcast_tx.clone();
        async move {
            loop {
                let mut connection = pool.acquire().await.unwrap();

                let session_data = match smartcard.get_card_details() {
                    Some(CardDetailsWithAuthInfo { card_details, .. })
                        if card_details.jurisdiction_code() == jurisdiction_code =>
                    {
                        let elections = db::get_elections(&mut connection).await.unwrap();
                        let pending_registration_requests =
                            db::get_pending_registration_requests(&mut connection)
                                .await
                                .unwrap();
                        let registrations = db::get_registrations(&mut connection).await.unwrap();
                        SessionData::Authenticated {
                            jurisdiction_code: jurisdiction_code.clone(),
                            elections,
                            pending_registration_requests,
                            registrations,
                        }
                    }
                    Some(_) => SessionData::Unauthenticated {
                        has_smartcard: true,
                    },
                    None => SessionData::Unauthenticated {
                        has_smartcard: false,
                    },
                };

                let _ = broadcast_tx.send(session_data);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    });

    router
        .route("/api/status", get(get_status))
        .route("/api/status-stream", get(get_status_stream))
        .route("/api/elections", get(get_elections))
        .route("/api/elections", post(create_election))
        .route("/api/registrations", post(create_registration))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            pool,
            smartcard,
            broadcast_tx,
        })
}

/// Runs an application built by `app::setup(…)`.
pub(crate) async fn run(app: Router, config: &Config) -> color_eyre::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
    tracing::info!("Server listening at http://{addr}/");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

fn distinct_until_changed<S: Stream>(stream: S) -> impl Stream<Item = S::Item>
where
    S::Item: Clone + PartialEq,
{
    let mut last = None;
    stream.filter(move |item| {
        let changed = last.as_ref() != Some(item);
        last = Some(item.clone());
        changed
    })
}

async fn get_status_stream(
    State(AppState { broadcast_tx, .. }): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let broadcast_rx = broadcast_tx.subscribe();

    let stream = distinct_until_changed(
        tokio_stream::wrappers::BroadcastStream::new(broadcast_rx).filter_map(Result::ok),
    )
    .map(|data| Event::default().json_data(data).unwrap())
    .map(Ok);

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn get_elections(State(AppState { pool, .. }): State<AppState>) -> impl IntoResponse {
    let mut connection = match pool.acquire().await {
        Ok(connection) => connection,
        Err(e) => {
            tracing::error!("error getting database connection: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error getting database connection" })),
            );
        }
    };

    let elections = match db::get_elections(&mut connection).await {
        Ok(elections) => elections,
        Err(e) => {
            tracing::error!("error getting elections from database: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error getting elections from database" })),
            );
        }
    };

    (StatusCode::OK, Json(json!({ "elections": elections })))
}

async fn create_election(
    State(AppState {
        pool, smartcard, ..
    }): State<AppState>,
    body: Bytes,
) -> impl IntoResponse {
    let jurisdiction_code = match smartcard.get_card_details() {
        Some(card_details) => card_details.card_details.jurisdiction_code(),
        None => {
            tracing::error!("no card details found");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "no card details found" })),
            );
        }
    };

    let election_definition = match ElectionDefinition::try_from(&body[..]) {
        Ok(election_definition) => election_definition,
        Err(e) => {
            tracing::error!("error parsing election definition: {e}");
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("error parsing election definition: {e}") })),
            );
        }
    };

    let mut connection = match pool.acquire().await {
        Ok(connection) => connection,
        Err(e) => {
            tracing::error!("error getting database connection: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error getting database connection" })),
            );
        }
    };

    let payload = Payload::Election(Election {
        jurisdiction_code,
        election_definition,
    });
    let serialized_payload = match serde_json::to_vec(&payload) {
        Ok(serialized_payload) => serialized_payload,
        Err(e) => {
            tracing::error!("error serializing payload: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error serializing payload" })),
            );
        }
    };

    let signed = match smartcard.sign(&serialized_payload, None) {
        Ok(signed) => signed,
        Err(e) => {
            tracing::error!("error signing payload: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error signing payload" })),
            );
        }
    };
    let certificates: Vec<u8> = match signed
        .cert_stack
        .iter()
        .map(|cert| cert.to_pem())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(certificates) => certificates.concat(),
        Err(e) => {
            tracing::error!("error converting certificates to PEM: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error converting certificates to PEM" })),
            );
        }
    };
    let signed_object = SignedObject {
        id: Uuid::new_v4(),
        payload: serialized_payload,
        certificates,
        signature: signed.data,
    };

    if let Err(e) = db::add_object(&mut connection, &signed_object).await {
        tracing::error!("error adding object to database: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "error adding object to database" })),
        );
    }

    (StatusCode::CREATED, Json(json!({ "id": signed_object.id })))
}

async fn create_registration(
    State(AppState {
        pool, smartcard, ..
    }): State<AppState>,
    Json(CreateRegistrationData {
        registration_request_id,
        election_id,
        ballot_style_id,
        precinct_id,
    }): Json<CreateRegistrationData>,
) -> impl IntoResponse {
    let jurisdiction_code = match smartcard.get_card_details() {
        Some(card_details) => card_details.card_details.jurisdiction_code(),
        None => {
            tracing::error!("no card details found");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "no card details found" })),
            );
        }
    };

    let mut connection = match pool.acquire().await {
        Ok(connection) => connection,
        Err(e) => {
            tracing::error!("error getting database connection: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error getting database connection" })),
            );
        }
    };

    let registration_request =
        match db::get_registration_request(&mut connection, registration_request_id).await {
            Ok(registration_request) => registration_request,
            Err(e) => {
                tracing::error!("error getting registration request from database: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "error getting registration request from database" })),
                );
            }
        };

    let payload = Payload::Registration(Registration {
        jurisdiction_code,
        common_access_card_id: registration_request.common_access_card_id,
        registration_request_object_id: registration_request_id,
        election_object_id: election_id,
        ballot_style_id,
        precinct_id,
    });
    let serialized_payload = match serde_json::to_vec(&payload) {
        Ok(serialized_payload) => serialized_payload,
        Err(e) => {
            tracing::error!("error serializing payload: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error serializing payload" })),
            );
        }
    };

    let signed = match smartcard.sign(&serialized_payload, None) {
        Ok(signed) => signed,
        Err(e) => {
            tracing::error!("error signing payload: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error signing payload" })),
            );
        }
    };
    let certificates: Vec<u8> = match signed
        .cert_stack
        .iter()
        .map(|cert| cert.to_pem())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(certificates) => certificates.concat(),
        Err(e) => {
            tracing::error!("error converting certificates to PEM: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "error converting certificates to PEM" })),
            );
        }
    };
    let signed_object = SignedObject {
        id: Uuid::new_v4(),
        payload: serialized_payload,
        certificates,
        signature: signed.data,
    };

    if let Err(e) = db::add_object(&mut connection, &signed_object).await {
        tracing::error!("error adding object to database: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "error adding object to database" })),
        );
    }

    (StatusCode::CREATED, Json(json!({ "id": signed_object.id })))
}
