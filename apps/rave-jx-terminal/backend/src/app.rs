//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use async_stream::try_stream;
use axum::extract::Query;
use axum::response::sse::{Event, KeepAlive};
use axum::response::{Response, Sse};
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Json, Router,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use futures_core::Stream;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use tokio::time::sleep;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;
use types_rs::election::ElectionDefinition;
use types_rs::rave::jx::AppData;
use types_rs::rave::{jx, ServerId};

use crate::config::{Config, MAX_REQUEST_SIZE};
use crate::db::{self, get_app_data};
use crate::smartcard::StatusGetter;

type AppState = (Config, PgPool, StatusGetter);

/// Prepares the application with all the routes. Run the application with
/// `app::run(…)` once you have it.
pub(crate) fn setup(pool: PgPool, config: Config, smartcard_status: StatusGetter) -> Router {
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
        .route("/api/smartcard-stream", get(get_smartcard_stream))
        .route("/api/jurisdictions", get(get_jurisdictions))
        .route("/api/elections", post(create_election))
        .route("/api/registrations", post(create_registration))
        .route("/api/auth/status", get(get_auth_status))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state((config, pool, smartcard_status))
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

#[derive(Debug, Deserialize)]
struct Scope {
    jurisdiction_id: ServerId,
}

async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

fn get_has_card(card_ctx: &pcsc::Context) -> bool {
    let mut readers_buf = [0; 2048];

    let Ok(mut readers) = card_ctx.list_readers(&mut readers_buf) else {
        return false;
    };

    let reader = match readers.next() {
        Some(reader) => reader,
        None => return false,
    };

    let card = match card_ctx.connect(reader, pcsc::ShareMode::Exclusive, pcsc::Protocols::ANY) {
        Ok(card) => card,
        Err(pcsc::Error::NoSmartcard) => {
            return false;
        }
        Err(err) => {
            return false;
        }
    };

    true
}

async fn get_smartcard_stream(
    State((_, _, smartcard_status)): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut last_status = None;

    Sse::new(try_stream! {
        loop {
            let status = Some(smartcard_status.get());

            if status != last_status {
                yield Event::default().json_data(&status).unwrap();
                last_status = status.clone();
            }

            sleep(Duration::from_millis(100)).await;
        }
    })
    .keep_alive(KeepAlive::default())
}

async fn get_status_stream(
    State((_, pool, _)): State<AppState>,
    Query(Scope { jurisdiction_id }): Query<Scope>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut last_app_data = jx::AppData::default();

    Sse::new(try_stream! {
        loop {
            let mut connection = pool.acquire().await.unwrap();
            let logged_in_app_data = get_app_data(&mut connection, jurisdiction_id).await.unwrap();

            match last_app_data {
                AppData::LoggedIn(data) if data != logged_in_app_data => {
                    yield Event::default().json_data(&logged_in_app_data).unwrap();
                    last_app_data = AppData::LoggedIn(logged_in_app_data);
                }
                _ => {},
            }

            sleep(Duration::from_secs(1)).await;
        }
    })
    .keep_alive(KeepAlive::default())
}

async fn get_jurisdictions(State((_, pool, _)): State<AppState>) -> impl IntoResponse {
    let mut connection = pool.acquire().await.map_err(into_internal_error)?;

    let jurisdictions = db::get_jurisdictions(&mut connection)
        .await
        .map_err(into_internal_error)?;

    Ok::<_, Response>(Json(json!({ "jurisdictions": jurisdictions })))
}

async fn create_election(
    State((config, pool, _)): State<AppState>,
    Query(Scope { jurisdiction_id }): Query<Scope>,
    election: String,
) -> impl IntoResponse {
    let election_definition: ElectionDefinition = election.parse().map_err(into_internal_error)?;
    let mut connection = pool.acquire().await.map_err(into_internal_error)?;

    db::add_election(
        &mut connection,
        &config,
        jurisdiction_id,
        election_definition,
    )
    .await
    .map_err(into_internal_error)?;

    Ok::<_, Response>(StatusCode::CREATED)
}

async fn create_registration(
    State((config, pool, _)): State<AppState>,
    registration: Json<jx::CreateRegistrationData>,
) -> impl IntoResponse {
    let ballot_style_id = &registration.ballot_style_id;
    let precinct_id = &registration.precinct_id;

    let mut connection = pool.acquire().await.map_err(into_internal_error)?;

    db::create_registration(
        &mut connection,
        &config,
        registration.registration_request_id,
        registration.election_id,
        precinct_id,
        ballot_style_id,
    )
    .await
    .map_err(into_internal_error)?;

    Ok::<_, Response>(StatusCode::CREATED)
}

async fn get_auth_status(State((_, _, card_ctx)): State<AppState>) -> impl IntoResponse {
    Ok::<_, Response>(Json(json!({ "authenticated": false })))
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
