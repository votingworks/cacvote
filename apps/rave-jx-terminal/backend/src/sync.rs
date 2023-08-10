use std::time::Duration;

use rocket_db_pools::Database;
use tokio::time::sleep;
use types_rs::rave::{RaveServerSyncInput, RaveServerSyncOutput};

use crate::{
    db::{self, Db},
    env::RAVE_URL,
};

pub(crate) async fn sync_periodically(
    rocket: rocket::Rocket<rocket::Build>,
) -> rocket::fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => {
            let conn = &**db;
            let Ok(mut db) = conn.acquire().await else {
                return Err(rocket);
            };

            tokio::spawn(async move {
                loop {
                    match sync(&mut db).await {
                        Ok(_) => {
                            info!("Successfully synced with RAVE server");
                        }
                        Err(e) => {
                            error!("Failed to sync with RAVE server: {}", e);
                        }
                    }
                    sleep(Duration::from_secs(5)).await;
                }
            });
            Ok(rocket)
        }
        None => Err(rocket),
    }
}

pub(crate) async fn sync(executor: &mut sqlx::PgConnection) -> color_eyre::eyre::Result<()> {
    debug!("Syncing with RAVE server");

    check_status(RAVE_URL.join("/api/status")?).await?;

    let sync_input = RaveServerSyncInput {
        last_synced_election_id: db::get_last_synced_election_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!("failed to get last synced election ID: {}", e))
            })?,
        last_synced_registration_request_id: db::get_last_synced_registration_request_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get last synced registration request ID: {}",
                    e
                ))
            })?,
        last_synced_registration_id: db::get_last_synced_registration_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!("failed to get last synced registration ID: {}", e))
            })?,
        last_synced_scanned_ballot_id: db::get_last_synced_scanned_ballot_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get last synced scanned ballot ID: {}",
                    e
                ))
            })?,
        last_synced_printed_ballot_id: db::get_last_synced_printed_ballot_id(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get last synced printed ballot ID: {}",
                    e
                ))
            })?,
        elections: db::get_elections_to_sync_to_rave_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get elections to sync to RAVE server: {}",
                    e
                ))
            })?,
        registration_requests: db::get_registration_requests_to_sync_to_rave_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get registration requests to sync to RAVE server: {}",
                    e
                ))
            })?,
        registrations: db::get_registrations_to_sync_to_rave_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get registrations to sync to RAVE server: {}",
                    e
                ))
            })?,
        printed_ballots: db::get_printed_ballots_to_sync_to_rave_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get printed ballots to sync to RAVE server: {}",
                    e
                ))
            })?,
        scanned_ballots: db::get_scanned_ballots_to_sync_to_rave_server(executor)
            .await
            .map_err(|e| {
                color_eyre::Report::msg(format!(
                    "failed to get scanned ballots to sync to RAVE server: {}",
                    e
                ))
            })?,
    };

    let sync_endpoint = RAVE_URL
        .join("/api/sync")
        .expect("failed to construct sync URL");
    let sync_output = request(sync_endpoint, &sync_input).await?;

    let RaveServerSyncOutput {
        admins,
        elections,
        registration_requests,
        registrations,
        printed_ballots,
        scanned_ballots,
    } = sync_output.clone();

    if let Err(e) = db::replace_admins_with_list_from_rave_server(executor, admins).await {
        error!("Failed to replace admins: {}", e);
    }

    for election in elections.into_iter() {
        let result = db::add_election_from_rave_server(executor, election).await;

        if let Err(e) = result {
            error!("Failed to insert election: {}", e);
        }
    }

    for registration_request in registration_requests.into_iter() {
        let result =
            db::add_or_update_registration_request_from_rave_server(executor, registration_request)
                .await;

        if let Err(e) = result {
            error!("Failed to insert or update registration request: {}", e);
        }
    }

    for registration in registrations.into_iter() {
        let result = db::add_or_update_registration_from_rave_server(executor, registration).await;

        if let Err(e) = result {
            error!("Failed to insert or update registration: {}", e);
        }
    }

    for printed_ballot in printed_ballots.into_iter() {
        let result =
            db::add_or_update_printed_ballot_from_rave_server(executor, printed_ballot).await;

        if let Err(e) = result {
            error!("Failed to insert or update printed ballot: {}", e);
        }
    }

    for scanned_ballot in scanned_ballots.into_iter() {
        let result =
            db::add_or_update_scanned_ballot_from_rave_server(executor, scanned_ballot).await;

        if let Err(e) = result {
            error!("Failed to insert or update scanned ballot: {}", e);
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
                "RAVE server responded with an error (status URL={}): {}",
                endpoint,
                e
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
