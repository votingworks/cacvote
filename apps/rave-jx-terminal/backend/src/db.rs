extern crate time;

use rocket::{fairing, Build, Rocket};
use rocket_db_pools::{sqlx, Database};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::Acquire;
use types_rs::cdf::cvr::Cvr;
use types_rs::election::ElectionDefinition;
use types_rs::rave::{client, ClientId, ServerId};

use crate::env::VX_MACHINE_ID;

#[derive(Database)]
#[database("sqlx")]
pub(crate) struct Db(rocket_db_pools::sqlx::PgPool);

pub(crate) async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match ::sqlx::migrate!("db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Admin {
    pub common_access_card_id: String,
    pub created_at: sqlx::types::time::OffsetDateTime,
}

impl From<Admin> for client::output::Admin {
    fn from(admin: Admin) -> Self {
        let Admin {
            common_access_card_id,
            created_at,
        } = admin;

        Self {
            common_access_card_id,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Election {
    pub id: ClientId,
    pub server_id: Option<ServerId>,
    pub client_id: ClientId,
    pub machine_id: String,
    pub definition: ElectionDefinition,
    pub election_hash: String,
    pub created_at: sqlx::types::time::OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScannedBallot {
    pub id: ClientId,
    pub server_id: Option<ServerId>,
    pub client_id: ClientId,
    pub machine_id: String,
    pub election_id: ClientId,
    pub cast_vote_record: Cvr,
    pub created_at: sqlx::types::time::OffsetDateTime,
}

pub(crate) async fn replace_admins_with_list_from_rave_server(
    executor: &mut sqlx::PgConnection,
    admins: Vec<client::output::Admin>,
) -> Result<(), sqlx::Error> {
    let mut txn = executor.begin().await?;

    sqlx::query!("DELETE FROM admins")
        .execute(&mut *txn)
        .await?;

    for admin in admins {
        add_admin_from_rave_server(&mut txn, admin).await?;
    }

    txn.commit().await?;

    Ok(())
}

pub(crate) async fn add_admin_from_rave_server(
    executor: &mut sqlx::PgConnection,
    admin: client::output::Admin,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO admins (
            common_access_card_id
        )
        VALUES ($1)
        ON CONFLICT (common_access_card_id)
        DO NOTHING
        "#,
        admin.common_access_card_id
    )
    .execute(executor)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn get_admins(
    executor: &mut sqlx::PgConnection,
) -> Result<Vec<Admin>, sqlx::Error> {
    sqlx::query_as!(
        Admin,
        r#"
        SELECT
            common_access_card_id,
            created_at
        FROM admins
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(&mut *executor)
    .await
}

pub(crate) async fn get_last_synced_election_id(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Option<ServerId>> {
    Ok(sqlx::query!(
        r#"
        SELECT server_id as "server_id: ServerId"
        FROM elections
        WHERE server_id IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *executor)
    .await?
    .map(|r| r.server_id)
    .flatten())
}

pub(crate) async fn get_last_synced_registration_request_id(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Option<ServerId>> {
    Ok(sqlx::query!(
        r#"
        SELECT server_id as "server_id: ServerId"
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
        SELECT server_id as "server_id: ServerId"
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
        SELECT server_id as "server_id: ServerId"
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
        server_id: Option<ServerId>,
        client_id: ClientId,
        machine_id: String,
        election: String,
        election_hash: String,
        created_at: sqlx::types::time::OffsetDateTime,
    }

    let records = match since_election {
        Some(election) => {
            sqlx::query_as!(
                ElectionRecord,
                r#"
                SELECT
                    id as "id: _",
                    server_id as "server_id: _",
                    client_id as "client_id: _",
                    machine_id,
                    election as "election: _",
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
                    id as "id: _",
                    server_id as "server_id: _",
                    client_id as "client_id: _",
                    machine_id,
                    election as "election: _",
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
                definition: record.election.parse()?,
                election_hash: record.election_hash,
                created_at: record.created_at,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) async fn add_election(
    executor: &mut sqlx::PgConnection,
    election: ElectionDefinition,
) -> color_eyre::Result<ClientId> {
    let client_id = ClientId::new();

    sqlx::query!(
        r#"
        INSERT INTO elections (
            id,
            client_id,
            machine_id,
            election_hash,
            election
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
        client_id.as_uuid(),
        client_id.as_uuid(),
        VX_MACHINE_ID.clone(),
        election.election_hash,
        election as _,
    )
    .execute(&mut *executor)
    .await?;

    Ok(client_id)
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
            election_hash,
            election
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (machine_id, client_id)
        DO UPDATE SET
            server_id = $2,
            election_hash = $5,
            election = $6
        "#,
        ClientId::new().as_uuid(),
        record.server_id.as_uuid(),
        record.client_id.as_uuid(),
        record.machine_id,
        record.election.election_hash,
        record.election as _
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
            common_access_card_id,
            registration_request_id,
            election_id,
            precinct_id,
            ballot_style_id,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (machine_id, client_id)
        DO UPDATE SET
            server_id = $2,
            common_access_card_id = $5,
            registration_request_id = $6,
            election_id = $7,
            precinct_id = $8,
            ballot_style_id = $9,
            created_at = $10
        "#,
        ClientId::new().as_uuid(),
        registration.server_id.as_uuid(),
        registration.client_id.as_uuid(),
        registration.machine_id,
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
            registration_id,
            cast_vote_record,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (client_id, machine_id)
        DO UPDATE SET
            server_id = $2,
            cast_vote_record = $7,
            created_at = $8
        "#,
        ClientId::new().as_uuid(),
        printed_ballot.server_id.as_uuid(),
        printed_ballot.client_id.as_uuid(),
        printed_ballot.machine_id,
        printed_ballot.common_access_card_id,
        registration_client_id.as_uuid(),
        Json(printed_ballot.cast_vote_record) as _,
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
        Json(scanned_ballot.cast_vote_record) as _,
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
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (client_id, machine_id)
        DO UPDATE SET
            server_id = $2,
            common_access_card_id = $5,
            given_name = $6,
            family_name = $7,
            address_line_1 = $8,
            address_line_2 = $9,
            city = $10,
            state = $11,
            postal_code = $12,
            state_id = $13,
            created_at = $14
        "#,
        ClientId::new().as_uuid(),
        registration_request.server_id.as_uuid(),
        registration_request.client_id.as_uuid(),
        registration_request.machine_id,
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
            election as "election!: String"
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
                election: e.election.parse()?,
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
            common_access_card_id,
            (SELECT client_id FROM registration_requests WHERE id = registration_request_id) as "registration_request_id!: ClientId",
            (SELECT client_id FROM elections WHERE id = election_id) as "election_id!: ClientId",
            precinct_id,
            ballot_style_id
        FROM registrations
        WHERE server_id IS NULL
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(&mut *executor)
    .await?;

    records
        .into_iter()
        .map(|r| {
            Ok(client::input::Registration {
                client_id: r.client_id,
                machine_id: r.machine_id,
                election_id: r.election_id,
                registration_request_id: r.registration_request_id,
                common_access_card_id: r.common_access_card_id,
                precinct_id: r.precinct_id,
                ballot_style_id: r.ballot_style_id,
            })
        })
        .collect()
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
            (SELECT client_id FROM registrations WHERE id = registration_id) as "registration_id!: ClientId",
            cast_vote_record as "cast_vote_record: String"
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
                registration_id: r.registration_id,
                cast_vote_record: serde_json::from_str(&r.cast_vote_record)?,
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
            election_id as "election_id: ClientId",
            cast_vote_record as "cast_vote_record: String",
            created_at
        FROM scanned_ballots
        WHERE server_id IS NULL
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&mut *executor)
    .await?;

    records
        .into_iter()
        .map(|r| {
            Ok(client::input::ScannedBallot {
                client_id: r.client_id,
                machine_id: r.machine_id,
                election_id: r.election_id,
                cast_vote_record: serde_json::from_str(r.cast_vote_record.as_str())?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

#[allow(dead_code)]
pub(crate) async fn add_scanned_ballot(
    executor: &mut sqlx::PgConnection,
    scanned_ballot: ScannedBallot,
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
        Json(cast_vote_record) as _,
        created_at
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
        SELECT server_id as "server_id: ServerId"
        FROM scanned_ballots
        WHERE server_id IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *executor)
    .await?
    .and_then(|r| r.server_id))
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScannedBallotStats {
    pub total: i64,
    pub pending: i64,
}

#[allow(dead_code)]
pub(crate) async fn get_scanned_ballot_stats(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<ScannedBallotStats> {
    sqlx::query_as!(
        ScannedBallotStats,
        r#"
        SELECT
            COUNT(*) as "total!: i64",
            COUNT(*) FILTER (WHERE server_id IS NULL) as "pending!: i64"
        FROM scanned_ballots
        "#
    )
    .fetch_one(&mut *executor)
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;
    use pretty_assertions::assert_eq;
    use time::OffsetDateTime;

    fn load_famous_names_election() -> ElectionDefinition {
        let election_json = include_str!("../tests/fixtures/electionFamousNames2021.json");
        election_json.parse().unwrap()
    }

    fn build_rave_server_registration_request() -> client::output::RegistrationRequest {
        client::output::RegistrationRequest {
            server_id: ServerId::new(),
            client_id: ClientId::new(),
            machine_id: "mark-terminal-001".to_owned(),
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
        election_definition: ElectionDefinition,
    ) -> client::output::Election {
        client::output::Election {
            server_id: ServerId::new(),
            client_id: ClientId::new(),
            machine_id: "mark-terminal-001".to_owned(),
            election_hash: election_definition.election_hash.clone(),
            election: election_definition,
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
            common_access_card_id: registration_request.common_access_card_id.clone(),
            registration_request_id: registration_request.server_id,
            election_id: election.server_id,
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
            registration_id: registration.registration_request_id,
            cast_vote_record,
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
            cast_vote_record,
            election_id: election.server_id,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_add_election_from_rave_server(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let mut db = pool.acquire().await?;

        let election_definition = load_famous_names_election();
        let record = build_rave_server_election(election_definition.clone());

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
    async fn test_add_election_to_be_synced(pool: sqlx::PgPool) -> sqlx::Result<()> {
        env::set_var("VX_MACHINE_ID", "rave-jx-test");

        let mut db = pool.acquire().await?;

        let election_definition = load_famous_names_election();

        let client_id = add_election(&mut db, election_definition.clone())
            .await
            .unwrap();

        let elections = get_elections_to_sync_to_rave_server(&mut db).await.unwrap();

        assert_eq!(elections.len(), 1);
        assert_eq!(elections[0].client_id, client_id);
        assert_eq!(
            elections[0].election.election_data,
            election_definition.election_data
        );

        Ok(())
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_add_everything_to_database(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let mut db = pool.acquire().await?;

        let election_definition = load_famous_names_election();
        let registration_request = build_rave_server_registration_request();
        let election = build_rave_server_election(election_definition.clone());
        let registration =
            build_rave_server_registration(&registration_request, &election, &election_definition);
        let printed_ballot = build_rave_server_printed_ballot(&registration, Cvr::default());
        let scanned_ballot = build_rave_server_scanned_ballot(&election, Cvr::default());

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
