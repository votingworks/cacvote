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
use sqlx::{self, postgres::PgPoolOptions, Connection, PgPool};
use tracing::Level;
use types_rs::cacvote::{JournalEntry, JurisdictionCode, SignedObject};
use uuid::Uuid;

use crate::config::Config;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

/// Sets up the database pool and runs any pending migrations, returning the
/// pool to be used by the app.
pub async fn setup(config: &Config) -> color_eyre::Result<PgPool> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up database").entered();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("db/migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn create_object(
    connection: &mut sqlx::PgConnection,
    object_type: &str,
    payload: &[u8],
    certificates: &[u8],
    signature: &[u8],
) -> color_eyre::Result<Uuid> {
    let mut txn = connection.begin().await?;

    // TODO: extract the jurisdiction code from the certificate
    let jurisdiction_code = JurisdictionCode::try_from("st.dev-jurisdiction").unwrap();

    // TODO: verify that the public key was signed by VX
    let object = sqlx::query!(
        r#"
        INSERT INTO objects (jurisdiction, object_type, payload, certificates, signature)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        jurisdiction_code.as_str(),
        object_type,
        payload,
        certificates,
        signature
    )
    .fetch_one(&mut *txn)
    .await?;

    tracing::debug!("Creating object with id {}", object.id);

    let journal_entry = sqlx::query!(
        r#"
        INSERT INTO journal_entries (object_id, jurisdiction, object_type, action)
        VALUES ($1, $2, $3, 'create')
        RETURNING id
        "#,
        object.id,
        jurisdiction_code.as_str(),
        object_type,
    )
    .fetch_one(&mut *txn)
    .await?;

    tracing::debug!("Creating journal entry with id {}", journal_entry.id);

    txn.commit().await?;

    tracing::debug!("Created object successfully");

    Ok(object.id)
}

pub async fn get_journal_entries(
    connection: &mut sqlx::PgConnection,
    since_journal_entry_id: Option<Uuid>,
) -> color_eyre::Result<Vec<types_rs::cacvote::JournalEntry>> {
    struct Record {
        id: Uuid,
        object_id: Uuid,
        jurisdiction: String,
        object_type: String,
        action: String,
        created_at: time::OffsetDateTime,
    }

    let entries = match since_journal_entry_id {
        Some(id) => {
            sqlx::query_as!(
                Record,
                r#"
                SELECT id, object_id, jurisdiction, object_type, action, created_at
                FROM journal_entries
                WHERE created_at > (SELECT created_at FROM journal_entries WHERE id = $1)
                ORDER BY created_at
                "#,
                id
            )
            .fetch_all(connection)
            .await?
        }
        None => {
            sqlx::query_as!(
                Record,
                r#"
                SELECT id, object_id, jurisdiction, object_type, action, created_at
                FROM journal_entries
                ORDER BY created_at
                "#,
            )
            .fetch_all(connection)
            .await?
        }
    };

    entries
        .into_iter()
        .map(|entry| {
            Ok(JournalEntry {
                id: entry.id,
                object_id: entry.object_id,
                jurisdiction: entry.jurisdiction.try_into().unwrap(),
                object_type: entry.object_type,
                action: entry.action,
                created_at: entry.created_at,
            })
        })
        .collect::<color_eyre::Result<Vec<_>>>()
}

pub async fn get_object_by_id(
    connection: &mut sqlx::PgConnection,
    object_id: Uuid,
) -> color_eyre::Result<Option<SignedObject>> {
    let object = sqlx::query!(
        r#"
        SELECT payload, certificates, signature
        FROM objects
        WHERE id = $1
        "#,
        object_id
    )
    .fetch_optional(connection)
    .await?;

    Ok(object.map(|object| SignedObject {
        payload: object.payload,
        certificates: object.certificates,
        signature: object.signature,
    }))
}