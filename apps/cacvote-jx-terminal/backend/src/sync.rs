//! CACvote Server synchronization utilities.

use cacvote_server_client::Client;
use tokio::time::sleep;
use types_rs::cacvote::JurisdictionCode;

use crate::{
    config::{Config, SYNC_INTERVAL},
    db,
};

/// Spawns an async loop that synchronizes with the CACvote Server on a fixed
/// schedule.
pub(crate) async fn sync_periodically(pool: &sqlx::PgPool, config: Config) {
    let mut connection = pool
        .acquire()
        .await
        .expect("failed to acquire database connection");

    let jurisdiction_code = config.jurisdiction_code().expect(
        "missing or invalid jurisdiction code in CA certificate; check that the CA certificate is valid and contains a jurisdiction code",
    );
    let mut client = Client::new(
        config.cacvote_url.clone(),
        config.ca_cert().expect("invalid CA certificate"),
        config.signer().expect("invalid signer"),
    );

    tokio::spawn(async move {
        loop {
            match sync(&mut connection, &mut client, &jurisdiction_code).await {
                Ok(_) => {
                    tracing::info!("Successfully synced with CACvote Server");
                }
                Err(e) => {
                    tracing::error!("Failed to sync with CACvote Server: {e}");
                }
            }
            sleep(SYNC_INTERVAL).await;
        }
    });
}

#[tracing::instrument(skip(executor, client), name = "Sync with CACvote Server")]
pub(crate) async fn sync(
    executor: &mut sqlx::PgConnection,
    client: &mut Client,
    jurisdiction_code: &JurisdictionCode,
) -> color_eyre::eyre::Result<()> {
    client.check_status().await?;

    push_objects(executor, client).await?;
    pull_journal_entries(executor, client, jurisdiction_code).await?;
    pull_objects(executor, client).await?;

    Ok(())
}

async fn pull_journal_entries(
    executor: &mut sqlx::PgConnection,
    client: &mut Client,
    jurisdiction_code: &JurisdictionCode,
) -> color_eyre::eyre::Result<()> {
    let latest_journal_entry_id = db::get_latest_journal_entry(executor)
        .await?
        .map(|entry| entry.id);
    tracing::debug!("fetching journal entries since {latest_journal_entry_id:?} in jurisdiction {jurisdiction_code}");
    let new_entries = client
        .get_journal_entries(latest_journal_entry_id.as_ref(), Some(jurisdiction_code))
        .await?;
    tracing::debug!(
        "fetched {count} new journal entries",
        count = new_entries.len()
    );
    db::add_journal_entries(executor, new_entries).await?;

    Ok(())
}

async fn push_objects(
    executor: &mut sqlx::PgConnection,
    client: &mut Client,
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
    client: &mut Client,
) -> color_eyre::eyre::Result<()> {
    let journal_entries = db::get_journal_entries_for_objects_to_pull(executor).await?;
    for journal_entry in journal_entries {
        match client.get_object_by_id(journal_entry.object_id).await? {
            Some(object) => {
                db::add_object_from_server(executor, &object).await?;
            }
            None => {
                tracing::warn!(
                    "Object with id {} not found on CACvote Server",
                    journal_entry.object_id
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use cacvote_server_client::{signer, PrivateKeySigner};
    use openssl::{pkey::PKey, x509::X509};
    use reqwest::Url;
    use tracing::Level;

    use crate::app;

    use super::*;

    const JURISDICTION_CODE: &str = "st.test-jurisdiction";

    async fn setup(pool: sqlx::PgPool) -> color_eyre::Result<Client> {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await?;
        let addr = listener.local_addr()?;
        let cacvote_url: Url = format!("http://{addr}").parse()?;
        let config = Config {
            cacvote_url: cacvote_url.clone(),
            database_url: "".to_owned(),
            machine_id: "".to_owned(),
            port: addr.port(),
            public_dir: None,
            log_level: Level::DEBUG,
            ca_cert: PathBuf::from("/not/real/path"),
            signer: signer::Description::File(PathBuf::from("/not/real/path")),
            eg_classpath: PathBuf::from("/not/real/path"),
        };

        tokio::spawn(async move {
            let app = app::setup(pool, config);
            axum::serve(listener, app).await.unwrap();
        });

        let cert = X509::from_pem(include_bytes!(
            "../../../../libs/auth/certs/dev/vx-cert-authority-cert.pem"
        ))
        .unwrap();
        let private_key = PKey::private_key_from_pem(include_bytes!(
            "../../../../libs/auth/certs/dev/vx-private-key.pem"
        ))
        .unwrap();
        let signer = PrivateKeySigner::new(private_key);
        Ok(Client::new(cacvote_url, cert, Box::new(signer)))
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_sync(pool: sqlx::PgPool) -> color_eyre::Result<()> {
        let mut connection = pool.acquire().await?;

        let mut client = setup(pool).await?;

        // TODO: actually test `sync`
        let _ = sync(
            &mut connection,
            &mut client,
            &JurisdictionCode::try_from(JURISDICTION_CODE).unwrap(),
        )
        .await;

        Ok(())
    }
}
