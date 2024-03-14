//! CACVote Server synchronization utilities.

use cacvote_server::client::Client;
use tokio::time::sleep;

use crate::{
    config::{Config, SYNC_INTERVAL},
    db,
};

/// Spawns an async loop that synchronizes with the CACVote Server on a fixed
/// schedule.
pub(crate) async fn sync_periodically(pool: &sqlx::PgPool, config: Config) {
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
    pull_objects(executor, client).await?;

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

async fn pull_objects(
    executor: &mut sqlx::PgConnection,
    client: &Client,
) -> color_eyre::eyre::Result<()> {
    let journal_entries = db::get_journal_entries_for_objects_to_pull(executor).await?;
    for journal_entry in journal_entries {
        match client.get_object_by_id(journal_entry.object_id).await? {
            Some(object) => {
                db::add_object_from_server(executor, &object).await?;
            }
            None => {
                tracing::warn!(
                    "Object with id {} not found on CACVote Server",
                    journal_entry.object_id
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{net::TcpListener, sync::Arc};

    use reqwest::Url;
    use tracing::Level;
    use types_rs::cacvote::SmartcardStatus;

    use crate::{
        app,
        smartcard::{DynStatusGetter, MockStatusGetterTrait},
    };

    use super::*;

    fn setup(pool: sqlx::PgPool, smartcard_status: DynStatusGetter) -> color_eyre::Result<Client> {
        let listener = TcpListener::bind("0.0.0.0:0")?;
        let addr = listener.local_addr()?;
        let cacvote_url: Url = format!("http://{addr}").parse()?;
        let config = Config {
            cacvote_url: cacvote_url.clone(),
            database_url: "".to_string(),
            machine_id: "".to_string(),
            port: addr.port(),
            public_dir: None,
            log_level: Level::DEBUG,
        };

        tokio::spawn(async move {
            let app = app::setup(pool, config, smartcard_status);
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        Ok(Client::new(cacvote_url))
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_sync(pool: sqlx::PgPool) -> color_eyre::Result<()> {
        let mut connection = pool.acquire().await?;

        let mut smartcard_status = MockStatusGetterTrait::new();
        smartcard_status
            .expect_get()
            .returning(|| SmartcardStatus::Card);

        let client = setup(pool, Arc::new(smartcard_status))?;

        sync(&mut connection, &client).await?;

        Ok(())
    }
}
