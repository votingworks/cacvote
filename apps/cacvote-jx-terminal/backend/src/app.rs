//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use async_stream::try_stream;
use axum::response::sse::{Event, KeepAlive};
use axum::response::Sse;
use axum::{extract::DefaultBodyLimit, routing::get, Router};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use futures_core::Stream;
use sqlx::PgPool;
use tokio::time::sleep;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::config::{Config, MAX_REQUEST_SIZE};
use crate::smartcard;

// type AppState = (Config, PgPool, smartcard::StatusGetter);
type AppState = (Config, PgPool, smartcard::DynStatusGetter);

/// Prepares the application with all the routes. Run the application with
/// `app::run(…)` once you have it.
pub(crate) fn setup(
    pool: PgPool,
    config: Config,
    smartcard_status: smartcard::DynStatusGetter,
) -> Router {
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

async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_status_stream(
    State((_, _pool, _smartcard_status)): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(try_stream! {
        loop {
            yield Event::default();
            sleep(Duration::from_secs(1)).await;
        }
    })
    .keep_alive(KeepAlive::default())
}
