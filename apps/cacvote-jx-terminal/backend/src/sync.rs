//! CACVote Server synchronization utilities.

use cacvote_server::client::Client;
use sqlx::PgPool;
use tokio::time::sleep;

use crate::{
    config::{Config, SYNC_INTERVAL},
    db,
};

/// Spawns an async loop that synchronizes with the CACVote Server on a fixed
/// schedule.
pub(crate) async fn sync_periodically(pool: &PgPool, config: Config) {
    let mut connection = pool
        .acquire()
        .await
        .expect("failed to acquire database connection");

    let client = Client::new(config.cacvote_url.clone());

    tokio::spawn(async move {
        loop {
            match sync(&mut connection, &client).await {
                Ok(_) => {
                    tracing::info!("Successfully synced with CACVote Server");
                }
                Err(e) => {
                    tracing::error!("Failed to sync with CACVote Server: {e}");
                }
            }
            sleep(SYNC_INTERVAL).await;
        }
    });
}

#[tracing::instrument(skip(executor, client), name = "Sync with CACVote Server")]
pub(crate) async fn sync(
    executor: &mut sqlx::PgConnection,
    client: &Client,
) -> color_eyre::eyre::Result<()> {
    client.check_status().await?;

    push_objects(executor, client).await?;
    pull_journal_entries(executor, client).await?;

    Ok(())
}

async fn pull_journal_entries(
    executor: &mut sqlx::PgConnection,
    client: &Client,
) -> color_eyre::eyre::Result<()> {
    let latest_journal_entry_id = db::get_latest_journal_entry(executor)
        .await?
        .map(|entry| entry.id);
    tracing::debug!("fetching journal entries since {latest_journal_entry_id:?}");
    let new_entries = client.get_journal_entries(latest_journal_entry_id).await?;
    tracing::debug!(
        "fetched {count} new journal entries",
        count = new_entries.len()
    );
    db::add_journal_entries(executor, new_entries).await?;

    Ok(())
}

async fn push_objects(
    executor: &mut sqlx::PgConnection,
    client: &Client,
) -> color_eyre::eyre::Result<()> {
    let objects = db::get_unsynced_objects(executor).await?;
    for object in objects {
        let object_id = client.create_object(object).await?;
        db::mark_object_synced(executor, object_id).await?;
    }

    Ok(())
}
