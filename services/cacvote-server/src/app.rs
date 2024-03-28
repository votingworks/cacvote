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
use types_rs::cacvote::{JournalEntry, JurisdictionCode, SignedObject};
use uuid::Uuid;

use crate::{
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
