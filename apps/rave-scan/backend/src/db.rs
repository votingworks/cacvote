//! Database access for the application.
//!
//! All direct use of [SQLx][`sqlx`] queries should be in this module. When
//! modifying this file, be sure to run `cargo sqlx prepare` in the application
//! root to regenerate the query metadata for offline builds.
//!
//! To enable `cargo sqlx prepare`, install it via `cargo install --locked
//! sqlx-cli`.

use std::str::FromStr;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::Level;
use types_rs::election::{ElectionDefinition, ElectionHash};
use types_rs::rave::{client, ClientId, ServerId};
use types_rs::scan::{BatchStats, ScannedBallotStats};

use crate::config::Config;

/// Sets up the database pool and runs any pending migrations, returning the
/// pool to be used by the app.
pub(crate) async fn setup(config: &Config) -> color_eyre::Result<PgPool> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up database").entered();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("db/migrations").run(&pool).await?;
    Ok(pool)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Election {
    pub(crate) id: ClientId,
    pub(crate) server_id: ServerId,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) definition: ElectionDefinition,
    pub(crate) election_hash: ElectionHash,
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScannedBallot {
    pub(crate) id: ClientId,
    pub(crate) server_id: Option<ServerId>,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) election_id: ClientId,
    pub(crate) cast_vote_record: Vec<u8>,
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

pub(crate) async fn get_last_synced_election_id(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Option<ServerId>> {
    Ok(sqlx::query!(
        r#"
        SELECT server_id as "server_id!: ServerId"
        FROM elections
        WHERE server_id IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *executor)
    .await?
    .map(|r| r.server_id))
}

pub(crate) async fn get_last_synced_registration_request_id(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Option<ServerId>> {
    Ok(sqlx::query!(
        r#"
        SELECT server_id as "server_id!: ServerId"
        FROM registration_requests
        WHERE server_id IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *executor)
    .await?
    .map(|r| r.server_id))
}

pub(crate) async fn get_last_synced_registration_id(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Option<ServerId>> {
    Ok(sqlx::query!(
        r#"
        SELECT server_id as "server_id!: ServerId"
        FROM registrations
        WHERE server_id IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *executor)
    .await?
    .map(|r| r.server_id))
}

pub(crate) async fn get_last_synced_printed_ballot_id(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Option<ServerId>> {
    Ok(sqlx::query!(
        r#"
        SELECT server_id as "server_id!: ServerId"
        FROM printed_ballots
        WHERE server_id IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *executor)
    .await?
    .map(|r| r.server_id))
}

#[allow(dead_code)]
pub(crate) async fn get_elections(
    executor: &mut sqlx::PgConnection,
    since_election_id: Option<ServerId>,
) -> Result<Vec<Election>, color_eyre::eyre::Error> {
    let since_election = match since_election_id {
        Some(id) => sqlx::query!(
            r#"
            SELECT created_at
            FROM elections
            WHERE id = $1
            "#,
            id.as_uuid(),
        )
        .fetch_optional(&mut *executor)
        .await
        .ok(),
        None => None,
    }
    .flatten();

    struct ElectionRecord {
        id: ClientId,
        server_id: ServerId,
        client_id: ClientId,
        machine_id: String,
        definition: String,
        election_hash: String,
        created_at: sqlx::types::time::OffsetDateTime,
    }

    let records = match since_election {
        Some(election) => {
            sqlx::query_as!(
                ElectionRecord,
                r#"
                SELECT
                    id as "id: ClientId",
                    server_id as "server_id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    definition as "definition: String",
                    election_hash,
                    created_at
                FROM elections
                WHERE created_at > $1
                ORDER BY created_at DESC
                "#,
                election.created_at
            )
            .fetch_all(&mut *executor)
            .await?
        }
        None => {
            sqlx::query_as!(
                ElectionRecord,
                r#"
                SELECT
                    id as "id: ClientId",
                    server_id as "server_id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    definition as "definition: String",
                    election_hash,
                    created_at
                FROM elections
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&mut *executor)
            .await?
        }
    };

    records
        .into_iter()
        .map(|record| {
            Ok::<Election, color_eyre::eyre::Error>(Election {
                id: record.id,
                server_id: record.server_id,
                client_id: record.client_id,
                machine_id: record.machine_id,
                definition: record.definition.parse()?,
                election_hash: ElectionHash::from_str(&record.election_hash)?,
                created_at: record.created_at,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) async fn add_or_update_jurisdiction_from_rave_server(
    executor: &mut sqlx::PgConnection,
    record: client::output::Jurisdiction,
) -> color_eyre::Result<ServerId> {
    sqlx::query!(
        r#"
        INSERT INTO jurisdictions (
            id,
            code,
            name,
            created_at
        )
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id)
        DO UPDATE SET
            code = $2,
            name = $3,
            created_at = $4
        "#,
        record.id.as_uuid(),
        record.code,
        record.name,
        record.created_at
    )
    .execute(executor)
    .await?;

    Ok(record.id)
}

pub(crate) async fn add_election_from_rave_server(
    executor: &mut sqlx::PgConnection,
    record: client::output::Election,
) -> color_eyre::Result<ClientId> {
    sqlx::query!(
        r#"
        INSERT INTO elections (
            id,
            server_id,
            client_id,
            machine_id,
            jurisdiction_id,
            election_hash,
            definition
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (machine_id, client_id)
        DO UPDATE SET
            server_id = $2,
            jurisdiction_id = $5,
            election_hash = $6,
            definition = $7
        "#,
        ClientId::new().as_uuid(),
        record.server_id.as_uuid(),
        record.client_id.as_uuid(),
        record.machine_id,
        record.jurisdiction_id.as_uuid(),
        record.definition.election_hash.as_str(),
        record.definition.election_data,
    )
    .execute(&mut *executor)
    .await?;

    let id = sqlx::query!(
        r#"
        SELECT id as "id: ClientId"
        FROM elections
        WHERE machine_id = $1 AND client_id = $2
        "#,
        record.machine_id,
        record.client_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?
    .id;

    Ok(id)
}

pub(crate) async fn add_or_update_registration_from_rave_server(
    executor: &mut sqlx::PgConnection,
    registration: client::output::Registration,
) -> color_eyre::Result<ClientId> {
    let registration_request_id = sqlx::query!(
        r#"
        SELECT id as "id: ClientId"
        FROM registration_requests
        WHERE server_id = $1
        "#,
        registration.registration_request_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?
    .id;

    let election_id = sqlx::query!(
        r#"
        SELECT id as "id: ClientId"
        FROM elections
        WHERE server_id = $1
        "#,
        registration.election_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?
    .id;

    sqlx::query!(
        r#"
        INSERT INTO registrations (
            id,
            server_id,
            client_id,
            machine_id,
            jurisdiction_id,
            common_access_card_id,
            registration_request_id,
            election_id,
            precinct_id,
            ballot_style_id,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (machine_id, client_id)
        DO UPDATE SET
            server_id = $2,
            jurisdiction_id = $5,
            common_access_card_id = $6,
            registration_request_id = $7,
            election_id = $8,
            precinct_id = $9,
            ballot_style_id = $10,
            created_at = $11
        "#,
        ClientId::new().as_uuid(),
        registration.server_id.as_uuid(),
        registration.client_id.as_uuid(),
        registration.machine_id,
        registration.jurisdiction_id.as_uuid(),
        registration.common_access_card_id,
        registration_request_id.as_uuid(),
        election_id.as_uuid(),
        registration.precinct_id,
        registration.ballot_style_id,
        registration.created_at
    )
    .execute(&mut *executor)
    .await?;

    let id = sqlx::query!(
        r#"
        SELECT id as "id: ClientId"
        FROM registrations
        WHERE machine_id = $1 AND client_id = $2
        "#,
        registration.machine_id,
        registration.client_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?
    .id;

    Ok(id)
}

pub(crate) async fn add_or_update_printed_ballot_from_rave_server(
    executor: &mut sqlx::PgConnection,
    printed_ballot: client::output::PrintedBallot,
) -> color_eyre::Result<ClientId> {
    let registration_client_id = sqlx::query!(
        r#"
        SELECT id as "id: ClientId"
        FROM registrations
        WHERE server_id = $1
        "#,
        printed_ballot.registration_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?
    .id;

    sqlx::query!(
        r#"
        INSERT INTO printed_ballots (
            id,
            server_id,
            client_id,
            machine_id,
            common_access_card_id,
            common_access_card_certificate,
            registration_id,
            cast_vote_record,
            cast_vote_record_signature,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (client_id, machine_id)
        DO UPDATE SET
            server_id = $2,
            cast_vote_record = $8,
            cast_vote_record_signature = $9,
            created_at = $10
        "#,
        ClientId::new().as_uuid(),
        printed_ballot.server_id.as_uuid(),
        printed_ballot.client_id.as_uuid(),
        printed_ballot.machine_id,
        printed_ballot.common_access_card_id,
        printed_ballot.common_access_card_certificate,
        registration_client_id.as_uuid(),
        printed_ballot.cast_vote_record,
        printed_ballot.cast_vote_record_signature,
        printed_ballot.created_at
    )
    .execute(&mut *executor)
    .await?;

    let id = sqlx::query!(
        r#"
        SELECT id as "id: ClientId"
        FROM printed_ballots
        WHERE machine_id = $1 AND client_id = $2
        "#,
        printed_ballot.machine_id,
        printed_ballot.client_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?
    .id;

    Ok(id)
}

pub(crate) async fn add_or_update_scanned_ballot_from_rave_server(
    executor: &mut sqlx::PgConnection,
    scanned_ballot: client::output::ScannedBallot,
) -> color_eyre::Result<ClientId> {
    let election_id = sqlx::query!(
        r#"
        SELECT id as "id: ClientId"
        FROM elections
        WHERE server_id = $1
        "#,
        scanned_ballot.election_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?
    .id;

    sqlx::query!(
        r#"
        INSERT INTO scanned_ballots (
            id,
            server_id,
            client_id,
            machine_id,
            election_id,
            cast_vote_record,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (client_id, machine_id)
        DO UPDATE SET
            server_id = $2,
            election_id = $5,
            cast_vote_record = $6,
            created_at = $7
        "#,
        ClientId::new().as_uuid(),
        scanned_ballot.server_id.as_uuid(),
        scanned_ballot.client_id.as_uuid(),
        scanned_ballot.machine_id,
        election_id.as_uuid(),
        scanned_ballot.cast_vote_record,
        scanned_ballot.created_at
    )
    .execute(&mut *executor)
    .await?;

    let id = sqlx::query!(
        r#"
        SELECT id as "id: ClientId"
        FROM scanned_ballots
        WHERE server_id = $1
        "#,
        scanned_ballot.server_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?
    .id;

    Ok(id)
}

pub(crate) async fn add_or_update_registration_request_from_rave_server(
    executor: &mut sqlx::PgConnection,
    registration_request: client::output::RegistrationRequest,
) -> color_eyre::Result<ClientId> {
    sqlx::query!(
        r#"
        INSERT INTO registration_requests (
            id,
            server_id,
            client_id,
            machine_id,
            jurisdiction_id,
            common_access_card_id,
            given_name,
            family_name,
            address_line_1,
            address_line_2,
            city,
            state,
            postal_code,
            state_id,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        ON CONFLICT (client_id, machine_id)
        DO UPDATE SET
            server_id = $2,
            jurisdiction_id = $5,
            common_access_card_id = $6,
            given_name = $7,
            family_name = $8,
            address_line_1 = $9,
            address_line_2 = $10,
            city = $11,
            state = $12,
            postal_code = $13,
            state_id = $14,
            created_at = $15
        "#,
        ClientId::new().as_uuid(),
        registration_request.server_id.as_uuid(),
        registration_request.client_id.as_uuid(),
        registration_request.machine_id,
        registration_request.jurisdiction_id.as_uuid(),
        registration_request.common_access_card_id,
        registration_request.given_name,
        registration_request.family_name,
        registration_request.address_line_1,
        registration_request.address_line_2,
        registration_request.city,
        registration_request.state,
        registration_request.postal_code,
        registration_request.state_id,
        registration_request.created_at
    )
    .execute(&mut *executor)
    .await?;

    let id = sqlx::query!(
        r#"
        SELECT id as "id: ClientId"
        FROM registration_requests
        WHERE server_id = $1
        "#,
        registration_request.server_id.as_uuid(),
    )
    .fetch_one(executor)
    .await?
    .id;

    Ok(id)
}

pub(crate) async fn get_registration_requests_to_sync_to_rave_server(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<client::input::RegistrationRequest>> {
    let records = sqlx::query!(
        r#"
        SELECT
            client_id as "client_id: ClientId",
            machine_id,
            jurisdiction_id as "jurisdiction_id: ServerId",
            common_access_card_id,
            given_name,
            family_name,
            address_line_1,
            address_line_2,
            city,
            state,
            postal_code,
            state_id
        FROM registration_requests
        WHERE server_id IS NULL
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(&mut *executor)
    .await?;

    Ok(records
        .into_iter()
        .map(|r| client::input::RegistrationRequest {
            client_id: r.client_id,
            machine_id: r.machine_id,
            jurisdiction_id: r.jurisdiction_id,
            common_access_card_id: r.common_access_card_id,
            given_name: r.given_name,
            family_name: r.family_name,
            address_line_1: r.address_line_1,
            address_line_2: r.address_line_2,
            city: r.city,
            state: r.state,
            postal_code: r.postal_code,
            state_id: r.state_id,
        })
        .collect())
}

pub(crate) async fn get_elections_to_sync_to_rave_server(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<client::input::Election>> {
    let records = sqlx::query!(
        r#"
        SELECT
            client_id as "client_id: ClientId",
            machine_id,
            jurisdiction_id as "jurisdiction_id: ServerId",
            definition as "definition: String"
        FROM elections
        WHERE server_id IS NULL
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(&mut *executor)
    .await?;

    records
        .into_iter()
        .map(|e| {
            Ok(client::input::Election {
                client_id: e.client_id,
                machine_id: e.machine_id,
                jurisdiction_id: e.jurisdiction_id,
                definition: e.definition.parse()?,
            })
        })
        .collect()
}

pub(crate) async fn get_registrations_to_sync_to_rave_server(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<client::input::Registration>> {
    let records = sqlx::query!(
        r#"
        SELECT
            client_id as "client_id: ClientId",
            machine_id,
            jurisdiction_id as "jurisdiction_id: ServerId",
            registration_request_id as "registration_request_id: ClientId",
            election_id as "election_id: ClientId",
            common_access_card_id,
            precinct_id,
            ballot_style_id
        FROM registrations
        WHERE server_id IS NULL
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(&mut *executor)
    .await?;

    Ok(records
        .into_iter()
        .map(|r| client::input::Registration {
            client_id: r.client_id,
            machine_id: r.machine_id,
            jurisdiction_id: r.jurisdiction_id,
            election_id: r.election_id,
            registration_request_id: r.registration_request_id,
            common_access_card_id: r.common_access_card_id,
            precinct_id: r.precinct_id,
            ballot_style_id: r.ballot_style_id,
        })
        .collect())
}

pub(crate) async fn get_printed_ballots_to_sync_to_rave_server(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<client::input::PrintedBallot>> {
    let records = sqlx::query!(
        r#"
        SELECT
            client_id as "client_id: ClientId",
            machine_id,
            common_access_card_id,
            common_access_card_certificate,
            registration_id as "registration_id: ClientId",
            cast_vote_record,
            cast_vote_record_signature
        FROM printed_ballots
        WHERE server_id IS NULL
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(&mut *executor)
    .await?;

    records
        .into_iter()
        .map(|r| {
            Ok(client::input::PrintedBallot {
                client_id: r.client_id,
                machine_id: r.machine_id,
                common_access_card_id: r.common_access_card_id,
                common_access_card_certificate: r.common_access_card_certificate,
                registration_id: r.registration_id,
                cast_vote_record: r.cast_vote_record,
                cast_vote_record_signature: r.cast_vote_record_signature,
            })
        })
        .collect()
}

pub(crate) async fn get_scanned_ballots_to_sync_to_rave_server(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<client::input::ScannedBallot>> {
    let records = sqlx::query!(
        r#"
        SELECT
            client_id as "client_id: ClientId",
            machine_id,
            (SELECT client_id FROM elections WHERE id = election_id) as "election_id!: ClientId",
            cast_vote_record,
            created_at
        FROM scanned_ballots
        WHERE server_id IS NULL
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&mut *executor)
    .await?;

    Ok(records
        .into_iter()
        .map(|r| client::input::ScannedBallot {
            client_id: r.client_id,
            machine_id: r.machine_id,
            election_id: r.election_id,
            cast_vote_record: r.cast_vote_record,
        })
        .collect::<Vec<_>>())
}

pub(crate) async fn start_batch(
    executor: &mut sqlx::PgConnection,
) -> Result<ClientId, sqlx::Error> {
    let batch_id = ClientId::new();

    sqlx::query!(
        r#"
        INSERT INTO batches (id, scanned_ballot_ids)
        VALUES ($1, $2)
        "#,
        batch_id.as_uuid(),
        &vec![],
    )
    .execute(executor)
    .await?;

    Ok(batch_id)
}

pub(crate) async fn end_batch(
    executor: &mut sqlx::PgConnection,
    batch_id: ClientId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE batches
        SET ended_at = NOW()
        WHERE id = $1
        "#,
        batch_id.as_uuid(),
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub(crate) async fn add_scanned_ballot(
    executor: &mut sqlx::PgConnection,
    scanned_ballot: ScannedBallot,
    batch_id: ClientId,
) -> Result<(), sqlx::Error> {
    let ScannedBallot {
        id,
        server_id,
        client_id,
        machine_id,
        election_id,
        cast_vote_record,
        created_at,
    } = scanned_ballot;
    sqlx::query!(
        r#"
        INSERT INTO scanned_ballots (
            id,
            server_id,
            client_id,
            machine_id,
            election_id,
            cast_vote_record,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        id.as_uuid(),
        server_id.map(|id| id.as_uuid()),
        client_id.as_uuid(),
        machine_id,
        election_id.as_uuid(),
        cast_vote_record,
        created_at
    )
    .execute(&mut *executor)
    .await?;

    sqlx::query!(
        r#"
        UPDATE batches
        SET scanned_ballot_ids = ARRAY_APPEND(scanned_ballot_ids, $1)
        WHERE id = $2
        "#,
        id.as_uuid(),
        batch_id.as_uuid(),
    )
    .execute(executor)
    .await?;

    Ok(())
}

pub(crate) async fn get_last_synced_scanned_ballot_id(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Option<ServerId>> {
    Ok(sqlx::query!(
        r#"
        SELECT server_id as "server_id!: ServerId"
        FROM scanned_ballots
        WHERE server_id IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *executor)
    .await?
    .map(|r| r.server_id))
}

pub(crate) async fn get_scanned_ballot_stats(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<ScannedBallotStats> {
    let batches = sqlx::query_as!(
        BatchStats,
        r#"
        SELECT
            id as "id: ClientId",
            COALESCE(ARRAY_LENGTH(scanned_ballot_ids, 1), 0) as "ballot_count!: _",
            CAST(
                (
                    SELECT COUNT(DISTINCT election_id)
                    FROM scanned_ballots
                    WHERE id IN (SELECT UNNEST(scanned_ballot_ids))
                ) AS int4
            ) AS "election_count!: _",
            CAST(
                (
                    SELECT COUNT(*)
                    FROM scanned_ballots
                    WHERE id IN (SELECT UNNEST(scanned_ballot_ids))
                    AND server_id IS NOT NULL
                ) AS int4
            ) AS "synced_count!: _",
            started_at,
            ended_at
        FROM batches
        ORDER BY started_at DESC
        "#
    )
    .fetch_all(executor)
    .await?;

    Ok(ScannedBallotStats { batches })
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use time::OffsetDateTime;
    use types_rs::cdf::cvr::Cvr;

    fn load_famous_names_election() -> ElectionDefinition {
        let election_json = include_str!("../tests/fixtures/electionFamousNames2021.json");
        election_json.parse().unwrap()
    }

    fn build_rave_server_jurisdiction() -> client::output::Jurisdiction {
        client::output::Jurisdiction {
            id: ServerId::new(),
            code: "st.test-jurisdiction".to_owned(),
            name: "Test Jurisdiction".to_owned(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    fn build_rave_server_registration_request(
        jurisdiction_id: ServerId,
    ) -> client::output::RegistrationRequest {
        client::output::RegistrationRequest {
            server_id: ServerId::new(),
            client_id: ClientId::new(),
            machine_id: "mark-terminal-001".to_owned(),
            jurisdiction_id,
            common_access_card_id: "0000000000".to_owned(),
            given_name: "John".to_owned(),
            family_name: "Doe".to_owned(),
            address_line_1: "123 Main St".to_owned(),
            address_line_2: None,
            city: "Anytown".to_owned(),
            state: "CA".to_owned(),
            postal_code: "95959".to_owned(),
            state_id: "CA-12345678".to_owned(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    fn build_rave_server_election(
        jurisdiction_id: ServerId,
        election_definition: ElectionDefinition,
    ) -> client::output::Election {
        client::output::Election {
            server_id: ServerId::new(),
            client_id: ClientId::new(),
            jurisdiction_id,
            machine_id: "mark-terminal-001".to_owned(),
            election_hash: election_definition.election_hash.clone(),
            definition: election_definition,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    fn build_rave_server_registration(
        registration_request: &client::output::RegistrationRequest,
        election: &client::output::Election,
        election_definition: &ElectionDefinition,
    ) -> client::output::Registration {
        let ballot_style = &election_definition.election.ballot_styles[0];

        client::output::Registration {
            server_id: registration_request.server_id,
            client_id: registration_request.client_id,
            machine_id: registration_request.machine_id.clone(),
            jurisdiction_id: election.jurisdiction_id,
            election_id: election.server_id,
            registration_request_id: registration_request.server_id,
            common_access_card_id: registration_request.common_access_card_id.clone(),
            precinct_id: ballot_style.precincts[0].to_string(),
            ballot_style_id: ballot_style.id.to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    fn build_rave_server_printed_ballot(
        registration: &client::output::Registration,
        cast_vote_record: Cvr,
    ) -> client::output::PrintedBallot {
        client::output::PrintedBallot {
            server_id: registration.server_id,
            client_id: registration.client_id,
            machine_id: registration.machine_id.clone(),
            common_access_card_id: registration.common_access_card_id.clone(),
            common_access_card_certificate: vec![],
            registration_id: registration.registration_request_id,
            cast_vote_record: serde_json::to_vec(&cast_vote_record).unwrap(),
            cast_vote_record_signature: vec![],
            created_at: OffsetDateTime::now_utc(),
        }
    }

    fn build_rave_server_scanned_ballot(
        election: &client::output::Election,
        cast_vote_record: Cvr,
    ) -> client::output::ScannedBallot {
        client::output::ScannedBallot {
            server_id: election.server_id,
            client_id: election.client_id,
            machine_id: election.machine_id.clone(),
            cast_vote_record: serde_json::to_vec(&cast_vote_record).unwrap(),
            election_id: election.server_id,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_add_election_from_rave_server(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let mut db = pool.acquire().await?;

        let election_definition = load_famous_names_election();
        let jurisdiction = build_rave_server_jurisdiction();
        let record = build_rave_server_election(jurisdiction.id, election_definition.clone());

        add_or_update_jurisdiction_from_rave_server(&mut db, jurisdiction)
            .await
            .unwrap();
        let client_id = add_election_from_rave_server(&mut db, record.clone())
            .await
            .unwrap();

        // insert again, should be idempotent
        let client_id2 = add_election_from_rave_server(&mut db, record.clone())
            .await
            .unwrap();

        assert_eq!(client_id, client_id2);

        Ok(())
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_add_everything_to_database(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let mut db = pool.acquire().await?;

        let election_definition = load_famous_names_election();
        let jurisdiction = build_rave_server_jurisdiction();
        let registration_request = build_rave_server_registration_request(jurisdiction.id);
        let election = build_rave_server_election(jurisdiction.id, election_definition.clone());
        let registration =
            build_rave_server_registration(&registration_request, &election, &election_definition);
        let printed_ballot = build_rave_server_printed_ballot(&registration, Cvr::default());
        let scanned_ballot = build_rave_server_scanned_ballot(&election, Cvr::default());

        add_or_update_jurisdiction_from_rave_server(&mut db, jurisdiction)
            .await
            .unwrap();

        add_election_from_rave_server(&mut db, election)
            .await
            .unwrap();

        add_or_update_registration_request_from_rave_server(&mut db, registration_request)
            .await
            .unwrap();

        add_or_update_registration_from_rave_server(&mut db, registration)
            .await
            .unwrap();

        add_or_update_printed_ballot_from_rave_server(&mut db, printed_ballot)
            .await
            .unwrap();

        add_or_update_scanned_ballot_from_rave_server(&mut db, scanned_ballot)
            .await
            .unwrap();

        Ok(())
    }
}
