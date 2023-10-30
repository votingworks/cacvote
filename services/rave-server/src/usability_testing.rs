//! # Usability Testing
//! This module contains code for setting up the database for usability testing.
//!
//! It is not intended to be used in production.

use sqlx::PgPool;
use tokio::time::sleep;
use types_rs::{
    election::ElectionDefinition,
    rave::{
        client::input::{Election, Jurisdiction, Registration},
        ClientId,
    },
};

use crate::{config::Config, db};

pub(crate) async fn setup(config: &Config, pool: &PgPool) -> color_eyre::Result<()> {
    if let Some(ref election_definition_path) = config.election_definition_path {
        let election_definition = std::fs::read_to_string(election_definition_path)?;
        let election_definition: ElectionDefinition = election_definition.parse()?;
        let mut txn = pool.begin().await?;

        let jurisdictions = db::get_jurisdictions(&mut txn).await?;
        let jurisdiction_id = match jurisdictions.first() {
            Some(jurisdiction) => jurisdiction.id,
            None => {
                crate::db::add_jurisdiction(
                    &mut txn,
                    Jurisdiction {
                        name: "Test Jurisdiction".to_owned(),
                    },
                )
                .await?
            }
        };
        crate::db::add_election(
            &mut txn,
            Election {
                client_id: ClientId::new(),
                machine_id: "rave-server".to_owned(),
                jurisdiction_id,
                definition: election_definition,
            },
        )
        .await?;
        txn.commit().await?;
    }

    if config.automatically_link_pending_registration_requests_with_latest_election {
        automatically_link_pending_registration_requests_with_latest_election_periodically(pool)
            .await;
    }

    Ok(())
}

const LINK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

pub(crate) async fn automatically_link_pending_registration_requests_with_latest_election_periodically(
    pool: &PgPool,
) {
    let mut connection = pool
        .acquire()
        .await
        .expect("failed to acquire database connection");

    tokio::spawn(async move {
        loop {
            match automatically_link_pending_registration_requests_with_latest_election(
                &mut connection,
            )
            .await
            {
                Ok(0) => {
                    tracing::info!("No pending registration requests to link");
                }
                Ok(n) => {
                    tracing::info!("Linked {n} pending registration request(s)");
                }
                Err(e) => {
                    tracing::error!("Failed to link pending registration requests: {e}");
                }
            }
            sleep(LINK_INTERVAL).await;
        }
    });
}

async fn automatically_link_pending_registration_requests_with_latest_election(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<usize> {
    let pending_registrations = sqlx::query!(
        r#"
        SELECT
            client_id AS "client_id: ClientId",
            common_access_card_id
        FROM registration_requests
        WHERE (SELECT COUNT(*) FROM registrations WHERE registration_requests.id = registrations.registration_request_id) = 0
        "#,
    )
    .fetch_all(&mut *executor)
    .await?
    .into_iter()
    .collect::<Vec<_>>();

    if pending_registrations.is_empty() {
        return Ok(0);
    }

    let elections = crate::db::get_elections(&mut *executor, None).await?;
    let latest_election = match elections
        .into_iter()
        .max_by_key(|election| election.created_at)
    {
        Some(election) => election,
        None => return Err(color_eyre::Report::msg("No elections found in database")),
    };

    let count = pending_registrations.len();
    let ballot_style = &latest_election.definition.election.ballot_styles[0];
    let precinct_id = &ballot_style.precincts[0];
    let ballot_style_id = &ballot_style.id;

    for pending_registration in pending_registrations {
        db::add_registration_from_client(
            &mut *executor,
            Registration {
                client_id: ClientId::new(),
                machine_id: "rave-server (automatic link)".to_owned(),
                common_access_card_id: pending_registration.common_access_card_id,
                jurisdiction_id: latest_election.jurisdiction_id,
                registration_request_id: pending_registration.client_id,
                election_id: latest_election.client_id,
                precinct_id: precinct_id.to_owned().to_string(),
                ballot_style_id: ballot_style_id.to_owned().to_string(),
            },
        )
        .await?;
    }

    Ok(count)
}
