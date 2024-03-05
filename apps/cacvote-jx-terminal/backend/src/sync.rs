//! CACVote Server synchronization utilities.

use sqlx::PgPool;
use tokio::time::sleep;
use tracing::Level;

use crate::config::{Config, SYNC_INTERVAL};

/// Spawns an async loop that synchronizes with the CACVote Server on a fixed
/// schedule.
pub(crate) async fn sync_periodically(pool: &PgPool, config: Config) {
    let mut connection = pool
        .acquire()
        .await
        .expect("failed to acquire database connection");

    tokio::spawn(async move {
        loop {
            match sync(&mut connection, &config).await {
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

pub(crate) async fn sync(
    _executor: &mut sqlx::PgConnection,
    config: &Config,
) -> color_eyre::eyre::Result<()> {
    let span = tracing::span!(Level::DEBUG, "Syncing with CACVote Server");
    let _enter = span.enter();

    check_status(config.cacvote_url.join("/api/status")?).await?;

    // TODO: Implement sync logic

    Ok(())
}

pub(crate) async fn check_status(endpoint: reqwest::Url) -> color_eyre::eyre::Result<()> {
    let client = reqwest::Client::new();
    client
        .get(endpoint.clone())
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            color_eyre::eyre::eyre!(
                "CACVote Server responded with an error (status URL={endpoint}): {e}",
            )
        })
        .map(|_| ())
}
