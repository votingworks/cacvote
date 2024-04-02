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
use color_eyre::eyre::bail;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgPool};
use tracing::Level;
use types_rs::cacvote::{JournalEntry, JurisdictionCode, SignedObject};
use uuid::Uuid;

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

pub(crate) async fn get_elections(
    connection: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Vec<types_rs::cacvote::Election>> {
    let objects = sqlx::query_as!(
        SignedObject,
        r#"
        SELECT
            id,
            payload,
            certificates,
            signature
        FROM objects
        WHERE object_type = 'Election'
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(connection)
    .await?;

    let mut elections = Vec::new();

    for object in objects {
        let payload = match object.try_to_inner() {
            Ok(payload) => {
                tracing::trace!("got object payload: {payload:?}");
                payload
            }
            Err(err) => {
                tracing::error!("unable to parse object payload: {err:?}");
                continue;
            }
        };

        if let types_rs::cacvote::Payload::Election(election) = payload {
            elections.push(election);
        }
    }

    Ok(elections)
}

#[tracing::instrument(skip(connection, object))]
pub async fn add_object_from_server(
    connection: &mut sqlx::PgConnection,
    object: &SignedObject,
) -> color_eyre::Result<Uuid> {
    if !object.verify()? {
        bail!("Unable to verify signature/certificates")
    }

    let Some(jurisdiction_code) = object.jurisdiction_code() else {
        bail!("No jurisdiction found");
    };

    let object_type = object.try_to_inner()?.object_type();

    sqlx::query!(
        r#"
        INSERT INTO objects (id, jurisdiction, object_type, payload, certificates, signature, server_synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, now())
        "#,
        &object.id,
        jurisdiction_code.as_str(),
        object_type,
        &object.payload,
        &object.certificates,
        &object.signature
    )
    .execute(connection)
    .await?;

    tracing::debug!("Created object with id {}", object.id);

    Ok(object.id)
}

#[tracing::instrument(skip(connection, object))]
pub async fn add_object(
    connection: &mut sqlx::PgConnection,
    object: &SignedObject,
) -> color_eyre::Result<Uuid> {
    if !object.verify()? {
        bail!("Unable to verify signature/certificates")
    }

    let Some(jurisdiction_code) = object.jurisdiction_code() else {
        bail!("No jurisdiction found");
    };

    let object_type = object.try_to_inner()?.object_type();

    sqlx::query!(
        r#"
        INSERT INTO objects (id, jurisdiction, object_type, payload, certificates, signature)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        &object.id,
        jurisdiction_code.as_str(),
        object_type,
        &object.payload,
        &object.certificates,
        &object.signature
    )
    .execute(connection)
    .await?;

    tracing::info!("Created object with id {}", object.id);

    Ok(object.id)
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
            entry.jurisdiction_code.as_str(),
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
            jurisdiction as "jurisdiction_code: JurisdictionCode",
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

pub(crate) async fn get_unsynced_objects(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Vec<SignedObject>> {
    Ok(sqlx::query_as!(
        SignedObject,
        r#"
        SELECT
            id,
            payload,
            certificates,
            signature
        FROM objects
        WHERE server_synced_at IS NULL
        "#,
    )
    .fetch_all(&mut *executor)
    .await?)
}

pub(crate) async fn mark_object_synced(
    executor: &mut sqlx::PgConnection,
    id: uuid::Uuid,
) -> color_eyre::eyre::Result<()> {
    sqlx::query!(
        r#"
        UPDATE objects
        SET server_synced_at = now()
        WHERE id = $1
        "#,
        id
    )
    .execute(&mut *executor)
    .await?;

    Ok(())
}

pub(crate) async fn get_journal_entries_for_objects_to_pull(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Vec<JournalEntry>> {
    Ok(sqlx::query_as!(
        JournalEntry,
        r#"
        SELECT
            id,
            object_id,
            jurisdiction as "jurisdiction_code: JurisdictionCode",
            object_type,
            action,
            created_at
        FROM journal_entries
        WHERE object_id IS NOT NULL
          AND object_type IN ('RegistrationRequest')
          AND object_id NOT IN (SELECT id FROM objects)
        "#,
    )
    .fetch_all(&mut *executor)
    .await?)
}
