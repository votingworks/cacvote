use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use config::{DATABASE_URL, MAX_REQUEST_SIZE, PORT, RAVE_URL, VX_MACHINE_ID};
use db::run_migrations;
use routes::*;
use sqlx::postgres::PgPoolOptions;

mod cac;
mod config;
mod db;
mod routes;
mod sync;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install().expect("could not install color_eyre");

    assert!(!VX_MACHINE_ID.is_empty(), "VX_MACHINE_ID must be set");
    assert!(!RAVE_URL.to_string().is_empty(), "RAVE_URL must be set");

    // database setup
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&DATABASE_URL)
        .await?;
    run_migrations(&pool).await?;

    // sync setup
    sync::sync_periodically(&pool).await;

    // application setup
    let app = Router::new()
        .route("/api/status", get(get_status))
        .route("/api/status-stream", get(get_status_stream))
        .route("/api/elections", post(create_election))
        .route("/api/registrations", post(create_registration))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .with_state(pool);

    // run the server
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), *PORT))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
