//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::time::Duration;

use async_stream::try_stream;
use axum::response::sse::{Event, KeepAlive};
use axum::response::{Response, Sse};
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Json, Router,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use futures_core::Stream;
use serde_json::json;
use sqlx::PgPool;
use tokio::time::sleep;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;
use types_rs::election::ElectionDefinition;
use types_rs::rave::jx;

use crate::config::{MAX_REQUEST_SIZE, PORT};
use crate::db::{self, get_app_data};

/// Prepares the application with all the routes. Run the application with
/// `app::run(…)` once you have it.
pub(crate) async fn setup(pool: PgPool) -> color_eyre::Result<Router> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up application").entered();

    let dist_path = Path::new("../frontend/dist");
    let _ = std::fs::create_dir_all(dist_path);

    Ok(Router::new()
        .fallback_service(
            ServeDir::new(dist_path)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(dist_path.join("index.html"))),
        )
        .route("/api/status", get(get_status))
        .route("/api/status-stream", get(get_status_stream))
        .route("/api/elections", post(create_election))
        .route("/api/registrations", post(create_registration))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state(pool))
}

/// Runs an application built by `app::setup(…)`.
pub(crate) async fn run(app: Router) -> color_eyre::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), *PORT);
    tracing::info!("Server listening at http://{addr}/");
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), *PORT))
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_status_stream(
    State(pool): State<PgPool>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut last_app_data = jx::AppData::default();

    Sse::new(try_stream! {
        loop {
            let mut connection = pool.acquire().await.unwrap();
            let app_data = get_app_data(&mut connection).await.unwrap();

            if app_data != last_app_data {
                yield Event::default().json_data(&app_data).unwrap();
                last_app_data = app_data;
            }

            sleep(Duration::from_secs(1)).await;
        }
    })
    .keep_alive(KeepAlive::default())
}

async fn create_election(State(pool): State<PgPool>, election: String) -> impl IntoResponse {
    let election_definition: ElectionDefinition = election.parse().map_err(into_internal_error)?;
    let mut connection = pool.acquire().await.map_err(into_internal_error)?;

    db::add_election(&mut connection, election_definition)
        .await
        .map_err(into_internal_error)?;

    Ok::<_, Response>(StatusCode::CREATED)
}

async fn create_registration(
    State(pool): State<PgPool>,
    registration: Json<jx::CreateRegistrationData>,
) -> impl IntoResponse {
    let ballot_style_id = &registration.ballot_style_id;
    let precinct_id = &registration.precinct_id;

    let mut connection = pool.acquire().await.map_err(into_internal_error)?;

    db::create_registration(
        &mut connection,
        registration.registration_request_id,
        registration.election_id,
        precinct_id,
        ballot_style_id,
    )
    .await
    .map_err(into_internal_error)?;

    Ok::<_, Response>(StatusCode::CREATED)
}

fn into_internal_error(e: impl std::fmt::Display) -> Response {
    tracing::error!("internal error: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "error": format!("{e}")
        })),
    )
        .into_response()
}
