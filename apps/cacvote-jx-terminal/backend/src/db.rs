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
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgPool};
use tracing::Level;
use types_rs::cacvote::{JournalEntry, JurisdictionCode};

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

#[tracing::instrument(skip(connection, entries))]
pub(crate) async fn add_journal_entries(
    connection: &mut sqlx::PgConnection,
    entries: Vec<JournalEntry>,
) -> color_eyre::eyre::Result<()> {
    let mut txn = connection.begin().await?;
    for entry in entries {
        sqlx::query!(
            r#"
            INSERT INTO journal_entries (id, object_id, jurisdiction, object_type, action, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO NOTHING
            "#,
            entry.id,
            entry.object_id,
            entry.jurisdiction.as_str(),
            entry.object_type,
            entry.action.as_str(),
            entry.created_at
        )
        .execute(&mut *txn)
        .await?;
    }
    txn.commit().await?;
    Ok(())
}

pub(crate) async fn get_latest_journal_entry(
    connection: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Option<JournalEntry>> {
    Ok(sqlx::query_as!(
        JournalEntry,
        r#"
        SELECT
            id,
            object_id,
            jurisdiction as "jurisdiction: JurisdictionCode",
            object_type,
            action,
            created_at
        FROM journal_entries
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut *connection)
    .await?)
}
