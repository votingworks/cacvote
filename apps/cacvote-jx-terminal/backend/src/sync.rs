//! CACVote Server synchronization utilities.

use sqlx::PgPool;
use tokio::time::sleep;
use tracing::Level;
use types_rs::cacvote::{RaveServerSyncInput, RaveServerSyncOutput};

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
    executor: &mut sqlx::PgConnection,
    config: &Config,
) -> color_eyre::eyre::Result<()> {
    let span = tracing::span!(Level::DEBUG, "Syncing with CACVote Server");
    let _enter = span.enter();

    check_status(config.cacvote_url.join("/api/status")?).await?;

    let sync_input = RaveServerSyncInput {
        last_synced_election_id: db::get_last_synced_election_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!("failed to get last synced election ID: {e}"))
            })?,
        last_synced_registration_request_id: db::get_last_synced_registration_request_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get last synced registration request ID: {e}"
                ))
            })?,
        last_synced_registration_id: db::get_last_synced_registration_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!("failed to get last synced registration ID: {e}"))
            })?,
        last_synced_scanned_ballot_id: db::get_last_synced_scanned_ballot_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!("failed to get last synced scanned ballot ID: {e}"))
            })?,
        last_synced_printed_ballot_id: db::get_last_synced_printed_ballot_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!("failed to get last synced printed ballot ID: {e}"))
            })?,
        elections: db::get_elections_to_sync_to_cacvote_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get elections to sync to CACVote Server: {e}"
                ))
            })?,
        registration_requests: db::get_registration_requests_to_sync_to_cacvote_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get registration requests to sync to CACVote Server: {e}"
                ))
            })?,
        registrations: db::get_registrations_to_sync_to_cacvote_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get registrations to sync to CACVote Server: {e}"
                ))
            })?,
        printed_ballots: db::get_printed_ballots_to_sync_to_cacvote_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get printed ballots to sync to CACVote Server: {e}"
                ))
            })?,
        scanned_ballots: db::get_scanned_ballots_to_sync_to_cacvote_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get scanned ballots to sync to CACVote Server: {e}"
                ))
            })?,
    };

    let sync_endpoint = config
        .cacvote_url
        .join("/api/sync")
        .expect("failed to construct sync URL");
    let sync_output = request(sync_endpoint, &sync_input).await?;

    let RaveServerSyncOutput {
        jurisdictions,
        elections,
        registration_requests,
        registrations,
        printed_ballots,
        scanned_ballots,
        ..
    } = sync_output.clone();

    for jurisdiction in jurisdictions {
        let result =
            db::add_or_update_jurisdiction_from_cacvote_server(executor, jurisdiction).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert or update jurisdiction: {e}");
        }
    }

    for election in elections {
        let result = db::add_election_from_cacvote_server(executor, election).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert election: {e}");
        }
    }

    for registration_request in registration_requests {
        let result = db::add_or_update_registration_request_from_cacvote_server(
            executor,
            registration_request,
        )
        .await;

        if let Err(e) = result {
            tracing::error!("Failed to insert or update registration request: {e}");
        }
    }

    for registration in registrations {
        let result =
            db::add_or_update_registration_from_cacvote_server(executor, registration).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert or update registration: {e}");
        }
    }

    for printed_ballot in printed_ballots {
        let result =
            db::add_or_update_printed_ballot_from_cacvote_server(executor, printed_ballot).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert or update printed ballot: {e}");
        }
    }

    for scanned_ballot in scanned_ballots {
        let result =
            db::add_or_update_scanned_ballot_from_cacvote_server(executor, scanned_ballot).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert or update scanned ballot: {e}");
        }
    }

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

pub(crate) async fn request(
    endpoint: reqwest::Url,
    sync_input: &RaveServerSyncInput,
) -> color_eyre::eyre::Result<RaveServerSyncOutput> {
    let client = reqwest::Client::new();
    Ok(client
        .post(endpoint.clone())
        .json(sync_input)
        .send()
        .await?
        .json::<RaveServerSyncOutput>()
        .await?)
}