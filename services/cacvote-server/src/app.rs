//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{
    extract::DefaultBodyLimit, http::StatusCode, response::IntoResponse, routing::get, Router,
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::config::{Config, MAX_REQUEST_SIZE};

/// Prepares the application to be run within an HTTP server.
///
/// Requires a [`PgPool`] from [`db::setup`]. Run the application with [`run`]
/// with the result of this function.
pub(crate) async fn setup(pool: PgPool) -> color_eyre::Result<Router> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up application").entered();
    Ok(Router::new()
        .route("/api/status", get(get_status))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state(pool))
}

/// Create and run an HTTP server using the provided application at the port
/// from [`config`][`super::config`].
pub(crate) async fn run(app: Router, config: &Config) -> color_eyre::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
    tracing::info!("Server listening at http://{addr}/");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

/// Always responds with a successful status. Used to check whether the server
/// is running.
pub(crate) async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}
