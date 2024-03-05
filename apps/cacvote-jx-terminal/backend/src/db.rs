//! Database access for the application.
//!
//! All direct use of [SQLx][`sqlx`] queries should be in this module. When
//! modifying this file, be sure to run `cargo sqlx prepare --workspace` in the
//! workspace root to regenerate the query metadata for offline builds.
//!
//! To enable `cargo sqlx prepare --workspace`, install it via `cargo install
//! --locked sqlx-cli`.

use std::time::Duration;

use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::Level;
use types_rs::cacvote::client::JurisdictionCode;
use types_rs::cacvote::jx;
use types_rs::cacvote::{ClientId, ServerId};
use types_rs::election::{BallotStyleId, ElectionDefinition, ElectionHash, PrecinctId};

use crate::config::Config;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

/// Sets up the database pool and runs any pending migrations, returning the
/// pool to be used by the app.
pub(crate) async fn setup(config: &Config) -> color_eyre::Result<PgPool> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up database").entered();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await?;
    tracing::debug!("Running database migrations");
    sqlx::migrate!("db/migrations").run(&pool).await?;
    Ok(pool)
}

pub(crate) async fn get_app_data(
    _executor: &mut sqlx::PgConnection,
    _jurisdiction_code: JurisdictionCode,
) -> color_eyre::Result<jx::LoggedInAppData> {
    Ok(jx::LoggedInAppData::default())
}

pub(crate) async fn add_election(
    _executor: &mut sqlx::PgConnection,
    _jurisdiction_code: JurisdictionCode,
    _election: ElectionDefinition,
    _return_address: &str,
) -> color_eyre::Result<ClientId> {
    todo!("add election to database");
}

pub(crate) async fn create_registration(
    _executor: &mut sqlx::PgConnection,
    _config: &Config,
    _registration_request_id: ClientId,
    _election_id: ClientId,
    _precinct_id: &PrecinctId,
    _ballot_style_id: &BallotStyleId,
) -> color_eyre::Result<ClientId> {
    todo!("add registration to database")
}
