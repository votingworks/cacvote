//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use async_stream::try_stream;
use auth_rs::card_details::CardDetailsWithAuthInfo;
use axum::body::Bytes;
use axum::response::sse::{Event, KeepAlive};
use axum::response::Sse;
use axum::routing::post;
use axum::Json;
use axum::{extract::DefaultBodyLimit, routing::get, Router};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use futures_core::Stream;
use serde_json::json;
use sqlx::PgPool;
use tokio::time::sleep;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;
use types_rs::cacvote::{AuthStatus, Election, Payload, SessionData, SignedObject};
use types_rs::election::ElectionDefinition;
use uuid::Uuid;

use crate::config::{Config, MAX_REQUEST_SIZE};
use crate::{db, smartcard};

// type AppState = (Config, PgPool, smartcard::StatusGetter);
type AppState = (Config, PgPool, smartcard::DynSmartcard);

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

    router
        .route("/api/status", get(get_status))
        .route("/api/status-stream", get(get_status_stream))
        .route("/api/elections", get(get_elections))
        .route("/api/elections", post(create_election))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state((config, pool, smartcard))
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

async fn get_status_stream(
    State((config, _, smartcard)): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(try_stream! {
        let mut last_card_details = None;

        loop {
            let new_card_details = smartcard.get_card_details();

            if new_card_details != last_card_details {
                last_card_details = new_card_details.clone();
                yield Event::default().json_data(SessionData {
                    auth_status: match new_card_details {
                        Some(CardDetailsWithAuthInfo { card_details, .. }) if card_details.jurisdiction_code() == config.jurisdiction_code => AuthStatus::Authenticated,
                        Some(_) => AuthStatus::UnauthenticatedInvalidCard,
                        None => AuthStatus::UnauthenticatedNoCard,
                    },
                    jurisdiction_code: Some(config.jurisdiction_code.clone()),
                }).unwrap();
            }

            sleep(Duration::from_millis(100)).await;
        }
    })
    .keep_alive(KeepAlive::default())
}

async fn get_elections(State((_, pool, _)): State<AppState>) -> impl IntoResponse {
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
    State((_, pool, smartcard)): State<AppState>,
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
