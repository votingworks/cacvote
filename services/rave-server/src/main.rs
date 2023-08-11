use std::{env, time::Duration};

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use config::{DEFAULT_PORT, MAX_REQUEST_SIZE};
use db::run_migrations;
use routes::*;
use sqlx::postgres::PgPoolOptions;

mod config;
mod db;
mod routes;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // database setup
    let db_connection_string = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_string)
        .await?;
    run_migrations(&pool).await?;

    // application setup
    let port: u32 = env::var("PORT").map_or_else(|_| Ok(DEFAULT_PORT), |port| port.parse())?;
    let app = Router::new()
        .route("/api/status", get(get_status))
        .route("/api/admins", post(create_admin))
        .route("/api/sync", post(do_sync))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .with_state(pool);

    // run the server
    axum::Server::bind(&format!("0.0.0.0:{port}").parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
