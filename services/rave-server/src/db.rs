//! Database access for the application.
//!
//! All direct use of [SQLx][`sqlx`] queries should be in this module. When
//! modifying this file, be sure to run `cargo sqlx prepare` in the application
//! root to regenerate the query metadata for offline builds.
//!
//! To enable `cargo sqlx prepare`, install it via `cargo install --locked
//! sqlx-cli`.

use std::{str::FromStr, time::Duration};

use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};
use sqlx::{self, postgres::PgPoolOptions, PgPool};
use tracing::Level;
use types_rs::{
    election::ElectionHash,
    rave::{
        client::{self, output::Jurisdiction},
        ClientId, ServerId,
    },
};

use crate::config::Config;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

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
pub(crate) struct Admin {
    pub(crate) machine_id: String,
    pub(crate) common_access_card_id: String,
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

impl From<Admin> for client::output::Admin {
    fn from(admin: Admin) -> Self {
        let Admin {
            machine_id,
            common_access_card_id,
            created_at,
        } = admin;

        Self {
            machine_id,
            common_access_card_id,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Election {
    pub(crate) id: ServerId,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) jurisdiction_id: ServerId,
    pub(crate) definition: types_rs::election::ElectionDefinition,
    pub(crate) election_hash: ElectionHash,
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

impl From<Election> for client::output::Election {
    fn from(election: Election) -> Self {
        let Election {
            id,
            client_id,
            machine_id,
            jurisdiction_id,
            definition,
            election_hash,
            created_at,
        } = election;

        Self {
            server_id: id,
            client_id,
            machine_id,
            jurisdiction_id,
            definition,
            election_hash,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RegistrationRequest {
    pub(crate) id: ServerId,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) jurisdiction_id: ServerId,
    pub(crate) common_access_card_id: String,
    pub(crate) given_name: String,
    pub(crate) family_name: String,
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

impl From<client::input::RegistrationRequest> for RegistrationRequest {
    fn from(request: client::input::RegistrationRequest) -> Self {
        let client::input::RegistrationRequest {
            jurisdiction_id,
            client_id,
            machine_id,
            common_access_card_id,
            given_name,
            family_name,
        } = request;

        Self {
            id: ServerId::new(),
            client_id,
            machine_id,
            jurisdiction_id,
            common_access_card_id,
            given_name,
            family_name,
            created_at: sqlx::types::time::OffsetDateTime::now_utc(),
        }
    }
}

impl From<RegistrationRequest> for client::output::RegistrationRequest {
    fn from(request: RegistrationRequest) -> Self {
        let RegistrationRequest {
            id,
            client_id,
            machine_id,
            jurisdiction_id,
            common_access_card_id,
            given_name,
            family_name,
            created_at,
        } = request;

        Self {
            server_id: id,
            client_id,
            machine_id,
            jurisdiction_id,
            common_access_card_id,
            given_name,
            family_name,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Registration {
    pub(crate) id: ServerId,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) common_access_card_id: String,
    pub(crate) jurisdiction_id: ServerId,
    pub(crate) registration_request_id: ServerId,
    pub(crate) election_id: ServerId,
    pub(crate) precinct_id: String,
    pub(crate) ballot_style_id: String,
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

impl From<Registration> for client::output::Registration {
    fn from(registration: Registration) -> Self {
        let Registration {
            id,
            client_id,
            machine_id,
            common_access_card_id,
            jurisdiction_id,
            registration_request_id,
            election_id,
            precinct_id,
            ballot_style_id,
            created_at,
        } = registration;

        Self {
            server_id: id,
            client_id,
            machine_id,
            common_access_card_id,
            jurisdiction_id,
            registration_request_id,
            election_id,
            precinct_id,
            ballot_style_id,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PrintedBallot {
    pub(crate) id: ServerId,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) common_access_card_id: String,
    #[serde(with = "Base64Standard")]
    pub(crate) common_access_card_certificate: Vec<u8>,
    pub(crate) registration_id: ServerId,
    #[serde(with = "Base64Standard")]
    pub(crate) cast_vote_record: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub(crate) cast_vote_record_signature: Vec<u8>,
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

impl From<PrintedBallot> for client::output::PrintedBallot {
    fn from(ballot: PrintedBallot) -> Self {
        let PrintedBallot {
            id,
            client_id,
            machine_id,
            common_access_card_id,
            common_access_card_certificate,
            registration_id,
            cast_vote_record,
            cast_vote_record_signature,
            created_at,
        } = ballot;

        Self {
            server_id: id,
            client_id,
            machine_id,
            common_access_card_id,
            common_access_card_certificate,
            registration_id,
            cast_vote_record,
            cast_vote_record_signature,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ScannedBallot {
    pub(crate) id: ServerId,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) election_id: ServerId,
    #[serde(with = "Base64Standard")]
    pub(crate) cast_vote_record: Vec<u8>,
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

impl From<ScannedBallot> for client::output::ScannedBallot {
    fn from(ballot: ScannedBallot) -> Self {
        let ScannedBallot {
            id,
            client_id,
            machine_id,
            election_id,
            cast_vote_record,
            created_at,
        } = ballot;

        Self {
            server_id: id,
            client_id,
            machine_id,
            election_id,
            cast_vote_record,
            created_at,
        }
    }
}

pub(crate) async fn add_jurisdiction(
    executor: &mut sqlx::PgConnection,
    jurisdiction: client::input::Jurisdiction,
) -> color_eyre::Result<ServerId> {
    let jurisdiction_id = ServerId::new();

    sqlx::query!(
        r#"
        INSERT INTO jurisdictions (id, name)
        VALUES ($1, $2)
        "#,
        jurisdiction_id.as_uuid(),
        jurisdiction.name
    )
    .execute(&mut *executor)
    .await?;

    Ok(jurisdiction_id)
}

pub(crate) async fn add_admin(
    executor: &mut sqlx::PgConnection,
    admin: client::input::Admin,
) -> color_eyre::Result<()> {
    let client::input::Admin {
        machine_id,
        common_access_card_id,
    } = admin;

    sqlx::query!(
        r#"
        INSERT INTO admins (machine_id, common_access_card_id)
        VALUES ($1, $2)
        ON CONFLICT (common_access_card_id) DO NOTHING
        "#,
        machine_id,
        common_access_card_id,
    )
    .execute(&mut *executor)
    .await?;

    Ok(())
}

pub(crate) async fn get_jurisdictions(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<Jurisdiction>> {
    sqlx::query_as!(
        Jurisdiction,
        r#"
        SELECT
            id,
            code,
            name,
            created_at
        FROM jurisdictions
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(&mut *executor)
    .await
    .map_err(Into::into)
}

pub(crate) async fn get_admins(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<Admin>> {
    sqlx::query_as!(
        Admin,
        r#"
        SELECT
            machine_id,
            common_access_card_id,
            created_at
        FROM admins
        ORDER BY created_at ASC
        "#
    )
    .fetch_all(&mut *executor)
    .await
    .map_err(Into::into)
}

pub(crate) async fn get_elections(
    executor: &mut sqlx::PgConnection,
    since_election_id: Option<ServerId>,
) -> color_eyre::Result<Vec<Election>> {
    struct ElectionRecord {
        id: ServerId,
        client_id: ClientId,
        machine_id: String,
        jurisdiction_id: ServerId,
        definition: Vec<u8>,
        election_hash: String,
        created_at: time::OffsetDateTime,
    }

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

    let records = match since_election {
        Some(election) => {
            sqlx::query_as!(
                ElectionRecord,
                r#"
                SELECT
                    id as "id: _",
                    client_id as "client_id: _",
                    machine_id,
                    jurisdiction_id,
                    definition,
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
                    client_id as "client_id: _",
                    machine_id,
                    jurisdiction_id,
                    definition,
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
        .map(|r| {
            Ok(Election {
                id: r.id,
                client_id: r.client_id,
                machine_id: r.machine_id,
                jurisdiction_id: r.jurisdiction_id,
                definition: r.definition.as_slice().try_into()?,
                election_hash: ElectionHash::from_str(&r.election_hash)?,
                created_at: r.created_at,
            })
        })
        .collect::<color_eyre::Result<Vec<_>>>()
}

pub(crate) async fn get_registration_requests(
    executor: &mut sqlx::PgConnection,
    since_registration_request_id: Option<ServerId>,
) -> color_eyre::Result<Vec<RegistrationRequest>> {
    let since_registration_request = match since_registration_request_id {
        Some(id) => {
            sqlx::query!(
                r#"
                SELECT created_at
                FROM registration_requests
                WHERE id = $1
                "#,
                id.as_uuid()
            )
            .fetch_optional(&mut *executor)
            .await?
        }
        None => None,
    };

    match since_registration_request {
        Some(registration_request) => sqlx::query_as!(
            RegistrationRequest,
            r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    jurisdiction_id as "jurisdiction_id: ServerId",
                    common_access_card_id,
                    given_name,
                    family_name,
                    created_at
                FROM registration_requests
                WHERE created_at > $1
                ORDER BY created_at DESC
                "#,
            registration_request.created_at
        )
        .fetch_all(&mut *executor)
        .await
        .map_err(Into::into),
        None => sqlx::query_as!(
            RegistrationRequest,
            r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    jurisdiction_id as "jurisdiction_id: ServerId",
                    common_access_card_id,
                    given_name,
                    family_name,
                    created_at
                FROM registration_requests
                ORDER BY created_at DESC
                "#,
        )
        .fetch_all(&mut *executor)
        .await
        .map_err(Into::into),
    }
}

pub(crate) async fn get_registrations(
    executor: &mut sqlx::PgConnection,
    since_registration_id: Option<ServerId>,
) -> color_eyre::Result<Vec<Registration>> {
    let since_registration = match since_registration_id {
        Some(registration_id) => sqlx::query!(
            r#"
        SELECT created_at
        FROM registrations
        WHERE id = $1
        "#,
            registration_id.as_uuid()
        )
        .fetch_optional(&mut *executor)
        .await
        .ok()
        .flatten(),
        None => None,
    };

    match since_registration {
        Some(registration) => sqlx::query_as!(
            Registration,
            r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    jurisdiction_id as "jurisdiction_id: ServerId",
                    common_access_card_id,
                    registration_request_id as "registration_request_id: ServerId",
                    election_id as "election_id: ServerId",
                    precinct_id,
                    ballot_style_id,
                    created_at
                FROM registrations
                WHERE created_at > $1
                ORDER BY created_at DESC
                "#,
            registration.created_at
        )
        .fetch_all(&mut *executor)
        .await
        .map_err(Into::into),
        None => sqlx::query_as!(
            Registration,
            r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    jurisdiction_id as "jurisdiction_id: ServerId",
                    common_access_card_id,
                    registration_request_id as "registration_request_id: ServerId",
                    election_id as "election_id: ServerId",
                    precinct_id,
                    ballot_style_id,
                    created_at
                FROM registrations
                ORDER BY created_at DESC
                "#,
        )
        .fetch_all(&mut *executor)
        .await
        .map_err(Into::into),
    }
}

pub(crate) async fn add_registration_request_from_client(
    executor: &mut sqlx::PgConnection,
    request: &client::input::RegistrationRequest,
) -> color_eyre::Result<ServerId> {
    let registration_request_id = ServerId::new();

    sqlx::query!(
        r#"
        INSERT INTO registration_requests (
            id,
            client_id,
            machine_id,
            jurisdiction_id,
            common_access_card_id,
            given_name,
            family_name
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        registration_request_id.as_uuid(),
        request.client_id.as_uuid(),
        request.machine_id,
        request.jurisdiction_id.as_uuid(),
        request.common_access_card_id,
        request.given_name,
        request.family_name,
    )
    .execute(executor)
    .await?;

    Ok(registration_request_id)
}

pub(crate) async fn add_election(
    executor: &mut sqlx::PgConnection,
    election: client::input::Election,
) -> Result<ServerId, color_eyre::eyre::Error> {
    let election_id = ServerId::new();
    let election_definition = election.definition;

    sqlx::query!(
        r#"
        INSERT INTO elections (
            id,
            jurisdiction_id,
            client_id,
            machine_id,
            election_hash,
            definition
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        election_id.as_uuid(),
        election.jurisdiction_id.as_uuid(),
        election.client_id.as_uuid(),
        election.machine_id,
        election_definition.election_hash.as_str(),
        election_definition.election_data
    )
    .execute(executor)
    .await?;

    Ok(election_id)
}

pub(crate) async fn add_registration_from_client(
    executor: &mut sqlx::PgConnection,
    registration: client::input::Registration,
) -> color_eyre::Result<ServerId> {
    let registration_id = ServerId::new();

    tracing::info!("Adding registration from client: {:?}", registration);
    sqlx::query!(
        r#"
        INSERT INTO registrations (
            id,
            client_id,
            machine_id,
            jurisdiction_id,
            election_id,
            registration_request_id,
            common_access_card_id,
            precinct_id,
            ballot_style_id
        )
        VALUES (
            $1, $2, $3, $4,
            (SELECT id FROM elections WHERE client_id = $5),
            (SELECT id FROM registration_requests WHERE client_id = $6),
            $7, $8, $9
        )
        "#,
        registration_id.as_uuid(),
        registration.client_id.as_uuid(),
        registration.machine_id,
        registration.jurisdiction_id.as_uuid(),
        registration.election_id.as_uuid(),
        registration.registration_request_id.as_uuid(),
        registration.common_access_card_id,
        registration.precinct_id,
        registration.ballot_style_id
    )
    .execute(&mut *executor)
    .await?;

    Ok(registration_id)
}

pub(crate) async fn add_printed_ballot_from_client(
    executor: &mut sqlx::PgConnection,
    ballot: client::input::PrintedBallot,
) -> color_eyre::Result<ServerId> {
    let ballot_id = ServerId::new();

    sqlx::query!(
        r#"
        INSERT INTO printed_ballots (
            id,
            client_id,
            machine_id,
            common_access_card_id,
            common_access_card_certificate,
            registration_id,
            cast_vote_record,
            cast_vote_record_signature
        )
        VALUES (
            $1, $2, $3, $4, $5,
            (SELECT id FROM registrations WHERE client_id = $6),
            $7, $8
        )
        "#,
        ballot_id.as_uuid(),
        ballot.client_id.as_uuid(),
        ballot.machine_id,
        ballot.common_access_card_id,
        ballot.common_access_card_certificate,
        ballot.registration_id.as_uuid(),
        ballot.cast_vote_record,
        ballot.cast_vote_record_signature
    )
    .execute(&mut *executor)
    .await?;

    Ok(ballot_id)
}

pub(crate) async fn get_printed_ballots(
    executor: &mut sqlx::PgConnection,
    since_ballot_id: Option<ServerId>,
) -> color_eyre::Result<Vec<PrintedBallot>> {
    let since_ballot = match since_ballot_id {
        Some(id) => sqlx::query!(
            r#"
            SELECT
                created_at
            FROM printed_ballots
            WHERE id = $1
            "#,
            id.as_uuid(),
        )
        .fetch_optional(&mut *executor)
        .await
        .ok()
        .flatten(),
        None => None,
    };

    struct PrintedBallotRecord {
        id: ServerId,
        client_id: ClientId,
        machine_id: String,
        common_access_card_id: String,
        common_access_card_certificate: Vec<u8>,
        registration_id: ServerId,
        cast_vote_record: Vec<u8>,
        cast_vote_record_signature: Vec<u8>,
        created_at: time::OffsetDateTime,
    }

    let records = match since_ballot {
        Some(ballot) => {
            sqlx::query_as!(
                PrintedBallotRecord,
                r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    common_access_card_id,
                    common_access_card_certificate,
                    registration_id as "registration_id: ServerId",
                    cast_vote_record,
                    cast_vote_record_signature,
                    created_at
                FROM printed_ballots
                WHERE created_at > $1
                ORDER BY created_at DESC
                "#,
                ballot.created_at
            )
            .fetch_all(&mut *executor)
            .await?
        }
        None => {
            sqlx::query_as!(
                PrintedBallotRecord,
                r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    common_access_card_id,
                    common_access_card_certificate,
                    registration_id as "registration_id: ServerId",
                    cast_vote_record,
                    cast_vote_record_signature,
                    created_at
                FROM printed_ballots
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&mut *executor)
            .await?
        }
    };

    records
        .into_iter()
        .map(|r| {
            Ok(PrintedBallot {
                id: r.id,
                client_id: r.client_id,
                machine_id: r.machine_id,
                common_access_card_id: r.common_access_card_id,
                common_access_card_certificate: r.common_access_card_certificate,
                registration_id: r.registration_id,
                cast_vote_record: r.cast_vote_record,
                cast_vote_record_signature: r.cast_vote_record_signature,
                created_at: r.created_at,
            })
        })
        .collect::<color_eyre::Result<Vec<_>>>()
}

pub(crate) async fn add_scanned_ballot_from_client(
    executor: &mut sqlx::PgConnection,
    ballot: client::input::ScannedBallot,
) -> color_eyre::Result<ServerId> {
    let ballot_id = ServerId::new();

    sqlx::query!(
        r#"
        INSERT INTO scanned_ballots (
            id,
            client_id,
            machine_id,
            election_id,
            cast_vote_record
        )
        VALUES (
            $1, $2, $3,
            (SELECT id FROM elections WHERE client_id = $4),
            $5
        )
        "#,
        ballot_id.as_uuid(),
        ballot.client_id.as_uuid(),
        ballot.machine_id,
        ballot.election_id.as_uuid(),
        ballot.cast_vote_record
    )
    .execute(&mut *executor)
    .await?;

    Ok(ballot_id)
}

pub(crate) async fn get_scanned_ballots(
    executor: &mut sqlx::PgConnection,
    since_ballot_id: Option<ServerId>,
) -> color_eyre::Result<Vec<ScannedBallot>> {
    let since_ballot = match since_ballot_id {
        Some(id) => sqlx::query!(
            r#"
            SELECT
                created_at
            FROM scanned_ballots
            WHERE id = $1
            "#,
            id.as_uuid(),
        )
        .fetch_optional(&mut *executor)
        .await
        .ok()
        .flatten(),
        None => None,
    };

    match since_ballot {
        Some(ballot) => sqlx::query_as!(
            ScannedBallot,
            r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    election_id as "election_id: ServerId",
                    cast_vote_record as "cast_vote_record: _",
                    created_at
                FROM scanned_ballots
                WHERE created_at > $1
                ORDER BY created_at DESC
                "#,
            ballot.created_at
        )
        .fetch_all(&mut *executor)
        .await
        .map_err(Into::into),
        None => sqlx::query_as!(
            ScannedBallot,
            r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    election_id as "election_id: ServerId",
                    cast_vote_record as "cast_vote_record: _",
                    created_at
                FROM scanned_ballots
                ORDER BY created_at DESC
                "#,
        )
        .fetch_all(&mut *executor)
        .await
        .map_err(Into::into),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_admins(pool: sqlx::PgPool) -> color_eyre::Result<()> {
        let mut db = pool.acquire().await?;

        add_admin(
            &mut db,
            client::input::Admin {
                machine_id: "machine-id".to_owned(),
                common_access_card_id: "1234567890".to_owned(),
            },
        )
        .await?;

        let admins = get_admins(&mut db).await?;

        assert_eq!(
            admins
                .into_iter()
                .map(|a| a.common_access_card_id)
                .collect::<Vec<_>>(),
            vec!["1234567890".to_owned()]
        );

        Ok(())
    }
}
