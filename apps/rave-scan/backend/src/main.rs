use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    time::Duration,
};

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use config::{DATABASE_URL, MAX_REQUEST_SIZE, PORT, VX_MACHINE_ID};
use db::run_migrations;
use routes::*;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

mod cards;
mod config;
mod db;
mod routes;
mod sync;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    assert!(!VX_MACHINE_ID.is_empty(), "VX_MACHINE_ID must be set");

    setup_logging()?;
    let pool = setup_database().await?;
    sync::sync_periodically(&pool).await;
    let app = setup_application(pool).await?;
    run_application(app).await
}

fn setup_logging() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let stdout_log = tracing_subscriber::fmt::layer().pretty();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    format!("{}=info", env!("CARGO_PKG_NAME").replace("-", "_")).parse()?,
                )
                .from_env_lossy(),
        )
        .with(stdout_log)
        .init();
    Ok(())
}

async fn setup_database() -> color_eyre::Result<PgPool> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up database").entered();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&DATABASE_URL)
        .await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

async fn setup_application(pool: PgPool) -> color_eyre::Result<Router> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up application").entered();

    let dist_path = Path::new("../frontend/dist");
    let _ = std::fs::create_dir_all(&dist_path);

    Ok(Router::new()
        .fallback_service(
            ServeDir::new(&dist_path)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(dist_path.join("index.html"))),
        )
        .route("/api/status", get(get_status))
        .route("/api/status-stream", get(get_status_stream))
        .route("/api/scan", post(do_scan))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state(pool))
}

async fn run_application(app: Router) -> color_eyre::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), *PORT);
    tracing::info!("Server listening at http://{addr}/");
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), *PORT))
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
