//! Database access for the application.
//!
//! All direct use of [SQLx][`sqlx`] queries should be in this module. When
//! modifying this file, be sure to run `cargo sqlx prepare --workspace` in the
//! workspace root to regenerate the query metadata for offline builds.
//!
//! To enable `cargo sqlx prepare --workspace`, install it via `cargo install
//! --locked sqlx-cli`.

use std::str::FromStr;
use std::time::Duration;

use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::Level;
use types_rs::cacvote::client::output::Jurisdiction;
use types_rs::cacvote::jx;
use types_rs::cacvote::{client, ClientId, ServerId};
use types_rs::election::{BallotStyleId, ElectionDefinition, ElectionHash, PrecinctId};
use uuid::Uuid;

use crate::cac::{verify_cast_vote_record, CertificateAuthority};
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
    tracing::debug!("Running database migrations");
    sqlx::migrate!("db/migrations").run(&pool).await?;
    Ok(pool)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Election {
    pub(crate) id: ClientId,
    pub(crate) jurisdiction_id: ServerId,
    pub(crate) server_id: Option<ServerId>,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) definition: ElectionDefinition,
    pub(crate) election_hash: ElectionHash,
    #[serde(with = "time::serde::iso8601")]
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RegistrationRequest {
    pub(crate) id: ClientId,
    pub(crate) server_id: ServerId,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) jurisdiction_id: ServerId,
    pub(crate) common_access_card_id: String,
    pub(crate) given_name: String,
    pub(crate) family_name: String,
    #[serde(with = "time::serde::iso8601")]
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Registration {
    pub(crate) id: ClientId,
    pub(crate) server_id: Option<ServerId>,
    pub(crate) client_id: ClientId,
    pub(crate) machine_id: String,
    pub(crate) jurisdiction_id: ServerId,
    pub(crate) common_access_card_id: String,
    pub(crate) registration_request_id: ClientId,
    pub(crate) election_id: ClientId,
    pub(crate) precinct_id: String,
    pub(crate) ballot_style_id: String,
    #[serde(with = "time::serde::iso8601")]
    pub(crate) created_at: sqlx::types::time::OffsetDateTime,
}

pub(crate) async fn get_app_data(
    executor: &mut sqlx::PgConnection,
    jurisdiction_code: String,
) -> color_eyre::Result<jx::LoggedInAppData> {
    let jurisdiction_id = get_jurisdiction_id_for_code(executor, &jurisdiction_code)
        .await?
        .ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "jurisdiction not found for jurisdiction code {jurisdiction_code}"
            )
        })?;
    let elections = get_elections(executor, None, jurisdiction_id).await?;
    let registration_requests = get_registration_requests(executor, jurisdiction_id).await?;
    let registrations = get_registrations(executor, jurisdiction_id).await?;
    let printed_ballots = get_printed_ballots(executor, jurisdiction_id).await?;

    Ok(jx::LoggedInAppData {
        registrations: registrations
            .into_iter()
            .map(|r| {
                let registration_request = registration_requests
                    .iter()
                    .find(|rr| rr.id == r.registration_request_id)
                    .ok_or_else(|| {
                        color_eyre::eyre::eyre!(
                            "registration request not found for registration {}",
                            r.id
                        )
                    })?;
                let election = elections
                    .iter()
                    .find(|e| e.id == r.election_id)
                    .ok_or_else(|| {
                        color_eyre::eyre::eyre!("election not found for registration {}", r.id)
                    })?;
                Ok(jx::Registration::new(
                    r.id,
                    r.server_id,
                    format!(
                        "{} {}",
                        registration_request.given_name, registration_request.family_name
                    ),
                    registration_request.common_access_card_id.clone(),
                    r.registration_request_id,
                    election.definition.election.title.clone(),
                    election.election_hash.clone(),
                    PrecinctId::from(r.precinct_id),
                    BallotStyleId::from(r.ballot_style_id),
                    r.created_at,
                ))
            })
            .collect::<color_eyre::Result<Vec<_>>>()?,
        elections: elections
            .into_iter()
            .map(|e| {
                jx::Election::new(
                    e.id,
                    e.server_id,
                    e.definition.election.title,
                    e.definition.election.date.date(),
                    e.definition.election.ballot_styles,
                    e.definition.election_hash,
                    e.created_at,
                )
            })
            .collect(),
        registration_requests: registration_requests
            .into_iter()
            .map(|rr| {
                jx::RegistrationRequest::new(
                    rr.id,
                    rr.server_id,
                    rr.common_access_card_id,
                    format!("{} {}", rr.given_name, rr.family_name),
                    rr.created_at,
                )
            })
            .collect(),
        printed_ballots,
    })
}

pub(crate) async fn get_jurisdiction_id_for_code(
    executor: &mut sqlx::PgConnection,
    jurisdiction_code: &str,
) -> color_eyre::Result<Option<ServerId>> {
    let jurisdiction = sqlx::query!(
        r#"
        SELECT id
        FROM jurisdictions
        WHERE code = $1
        "#,
        jurisdiction_code
    )
    .fetch_optional(executor)
    .await?;

    Ok(jurisdiction.map(|j| j.id.into()))
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

pub(crate) async fn get_jurisdictions(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<Jurisdiction>> {
    Ok(sqlx::query_as!(
        Jurisdiction,
        r#"
        SELECT
            id as "id: _",
            code,
            name,
            created_at
        FROM jurisdictions
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(executor)
    .await?)
}

#[allow(dead_code)]
pub(crate) async fn get_elections(
    executor: &mut sqlx::PgConnection,
    since_election_id: Option<ServerId>,
    jurisdiction_id: ServerId,
) -> Result<Vec<Election>, color_eyre::eyre::Error> {
    let since_election = match since_election_id {
        Some(id) => sqlx::query!(
            r#"
            SELECT created_at
            FROM elections
            WHERE id = $1
              AND jurisdiction_id = $2
            "#,
            id.as_uuid(),
            jurisdiction_id.as_uuid(),
        )
        .fetch_optional(&mut *executor)
        .await
        .ok(),
        None => None,
    }
    .flatten();

    struct ElectionRecord {
        // TODO: use ServerId and ClientId
        id: Uuid,
        jurisdiction_id: Uuid,
        server_id: Option<Uuid>,
        client_id: Uuid,
        machine_id: String,
        definition: Vec<u8>,
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
                    jurisdiction_id as "jurisdiction_id: _",
                    server_id as "server_id: _",
                    client_id as "client_id: _",
                    machine_id,
                    definition,
                    election_hash,
                    created_at
                FROM elections
                WHERE created_at > $1
                  AND jurisdiction_id = $2
                ORDER BY created_at DESC
                "#,
                election.created_at,
                jurisdiction_id.as_uuid(),
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
                    jurisdiction_id as "jurisdiction_id: _",
                    server_id as "server_id: _",
                    client_id as "client_id: _",
                    machine_id,
                    definition,
                    election_hash,
                    created_at
                FROM elections
                WHERE jurisdiction_id = $1
                ORDER BY created_at DESC
                "#,
                jurisdiction_id.as_uuid(),
            )
            .fetch_all(&mut *executor)
            .await?
        }
    };

    records
        .into_iter()
        .map(|record| {
            Ok(Election {
                id: record.id.into(),
                jurisdiction_id: record.jurisdiction_id.into(),
                server_id: record.server_id.map(Into::into),
                client_id: record.client_id.into(),
                machine_id: record.machine_id,
                definition: String::from_utf8(record.definition)?.parse()?,
                election_hash: ElectionHash::from_str(record.election_hash.as_str())?,
                created_at: record.created_at,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) async fn add_election(
    executor: &mut sqlx::PgConnection,
    config: &Config,
    jurisdiction_id: ServerId,
    election: ElectionDefinition,
    return_address: &str,
) -> color_eyre::Result<ClientId> {
    let client_id = ClientId::new();

    sqlx::query!(
        r#"
        INSERT INTO elections (
            id,
            jurisdiction_id,
            client_id,
            machine_id,
            election_hash,
            definition,
            return_address
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        client_id.as_uuid(),
        jurisdiction_id.as_uuid(),
        client_id.as_uuid(),
        config.machine_id.clone(),
        election.election_hash.as_str(),
        election.election_data,
        return_address,
    )
    .execute(&mut *executor)
    .await?;

    Ok(client_id)
}

pub(crate) async fn get_registration_requests(
    executor: &mut sqlx::PgConnection,
    jurisdiction_id: ServerId,
) -> color_eyre::Result<Vec<RegistrationRequest>> {
    sqlx::query_as!(
        RegistrationRequest,
        r#"
        SELECT
            id as "id: _",
            server_id as "server_id: _",
            client_id as "client_id: _",
            jurisdiction_id as "jurisdiction_id: _",
            machine_id,
            common_access_card_id,
            given_name,
            family_name,
            created_at
        FROM registration_requests
        WHERE jurisdiction_id = $1
        ORDER BY created_at DESC
        "#,
        jurisdiction_id.as_uuid(),
    )
    .fetch_all(&mut *executor)
    .await
    .map_err(Into::into)
}

pub(crate) async fn get_registrations(
    executor: &mut sqlx::PgConnection,
    jurisdiction_id: ServerId,
) -> color_eyre::Result<Vec<Registration>> {
    sqlx::query_as!(
        Registration,
        r#"
        SELECT
            id as "id: _",
            server_id as "server_id: _",
            client_id as "client_id: _",
            machine_id,
            jurisdiction_id as "jurisdiction_id: _",
            common_access_card_id,
            registration_request_id as "registration_request_id: _",
            election_id as "election_id: _",
            precinct_id,
            ballot_style_id,
            created_at
        FROM registrations
        WHERE jurisdiction_id = $1
        ORDER BY created_at DESC
        "#,
        jurisdiction_id.as_uuid(),
    )
    .fetch_all(&mut *executor)
    .await
    .map_err(Into::into)
}

pub(crate) async fn get_printed_ballots(
    executor: &mut sqlx::PgConnection,
    jurisdiction_id: ServerId,
) -> color_eyre::Result<Vec<jx::PrintedBallot>> {
    let records = sqlx::query!(
        r#"
        SELECT
            id,
            server_id,
            registration_id,
            common_access_card_certificate,
            (
                SELECT election_id
                FROM registrations
                WHERE registrations.id = registration_id
                  AND jurisdiction_id = $1
            ),
            (
                SELECT precinct_id
                FROM registrations
                WHERE registrations.id = registration_id
                  AND jurisdiction_id = $1
            ),
            (
                SELECT ballot_style_id
                FROM registrations
                WHERE registrations.id = registration_id
                  AND jurisdiction_id = $1
            ),
            cast_vote_record,
            cast_vote_record_signature,
            created_at
        FROM printed_ballots
        ORDER BY created_at DESC
        "#,
        jurisdiction_id.as_uuid(),
    )
    .fetch_all(&mut *executor)
    .await?;

    records
        .into_iter()
        .map(|record| {
            let verification_status = verify_cast_vote_record(
                &record.common_access_card_certificate,
                &record.cast_vote_record,
                &record.cast_vote_record_signature,
                CertificateAuthority::DodTest,
            );

            Ok(jx::PrintedBallot {
                id: record.id.into(),
                server_id: ServerId::from(record.server_id),
                registration_id: record.registration_id.into(),
                election_id: record.election_id.map(Into::into).ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "election_id is null for registration_id {}",
                        record.registration_id
                    )
                })?,
                precinct_id: record.precinct_id.map(PrecinctId::from).ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "precinct_id is null for registration_id {}",
                        record.registration_id
                    )
                })?,
                ballot_style_id: record.ballot_style_id.map(BallotStyleId::from).ok_or_else(
                    || {
                        color_eyre::eyre::eyre!(
                            "ballot_style_id is null for registration_id {}",
                            record.registration_id
                        )
                    },
                )?,
                cast_vote_record: record.cast_vote_record,
                cast_vote_record_signature: record.cast_vote_record_signature,
                verification_status,
                created_at: record.created_at,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) async fn create_registration(
    executor: &mut sqlx::PgConnection,
    config: &Config,
    registration_request_id: ClientId,
    election_id: ClientId,
    precinct_id: &PrecinctId,
    ballot_style_id: &BallotStyleId,
) -> color_eyre::Result<ClientId> {
    let registration_request = sqlx::query!(
        r#"
        SELECT
            common_access_card_id,
            jurisdiction_id as "jurisdiction_id: ServerId"
        FROM registration_requests
        WHERE id = $1
        "#,
        registration_request_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?;

    let election = sqlx::query!(
        r#"
        SELECT
            jurisdiction_id as "jurisdiction_id: ServerId"
        FROM elections
        WHERE id = $1
        "#,
        election_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?;

    if registration_request.jurisdiction_id != election.jurisdiction_id {
        return Err(color_eyre::eyre::eyre!(
            "registration request jurisdiction ID {} does not match election jurisdiction ID {}",
            registration_request.jurisdiction_id,
            election.jurisdiction_id
        ));
    }

    let registration_id = ClientId::new();

    sqlx::query!(
        r#"
        INSERT INTO registrations (
            id,
            client_id,
            machine_id,
            common_access_card_id,
            registration_request_id,
            election_id,
            jurisdiction_id,
            precinct_id,
            ballot_style_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        registration_id.as_uuid(),
        registration_id.as_uuid(),
        config.machine_id.clone(),
        registration_request.common_access_card_id,
        registration_request_id.as_uuid(),
        election_id.as_uuid(),
        registration_request.jurisdiction_id.as_uuid(),
        precinct_id.to_string(),
        ballot_style_id.to_string(),
    )
    .execute(&mut *executor)
    .await?;

    let registration_id = sqlx::query!(
        r#"
        SELECT
            id as "id: ClientId"
        FROM registrations
        WHERE registration_request_id = $1
          AND election_id = $2
          AND jurisdiction_id = $3
        "#,
        registration_request_id.as_uuid(),
        election_id.as_uuid(),
        registration_request.jurisdiction_id.as_uuid(),
    )
    .fetch_one(&mut *executor)
    .await?
    .id;

    Ok(registration_id)
}

pub(crate) async fn add_or_update_jurisdiction_from_cacvote_server(
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
    .execute(&mut *executor)
    .await?;

    Ok(record.id)
}

pub(crate) async fn add_election_from_cacvote_server(
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
            definition,
            return_address
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (machine_id, client_id)
        DO UPDATE SET
            server_id = $2,
            jurisdiction_id = $5,
            election_hash = $6,
            definition = $7,
            return_address = $8
        "#,
        ClientId::new().as_uuid(),
        record.server_id.as_uuid(),
        record.client_id.as_uuid(),
        record.machine_id,
        record.jurisdiction_id.as_uuid(),
        record.definition.election_hash.as_str(),
        record.definition as _,
        record.return_address,
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

pub(crate) async fn add_or_update_registration_from_cacvote_server(
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
            election_id,
            registration_request_id,
            common_access_card_id,
            precinct_id,
            ballot_style_id,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (machine_id, client_id)
        DO UPDATE SET
            server_id = $2,
            jurisdiction_id = $5,
            election_id = $6,
            registration_request_id = $7,
            common_access_card_id = $8,
            precinct_id = $9,
            ballot_style_id = $10,
            created_at = $11
        "#,
        ClientId::new().as_uuid(),
        registration.server_id.as_uuid(),
        registration.client_id.as_uuid(),
        registration.machine_id,
        registration.jurisdiction_id.as_uuid(),
        election_id.as_uuid(),
        registration_request_id.as_uuid(),
        registration.common_access_card_id,
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

pub(crate) async fn add_or_update_printed_ballot_from_cacvote_server(
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
            common_access_card_id = $5,
            common_access_card_certificate = $6,
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

pub(crate) async fn add_or_update_registration_request_from_cacvote_server(
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
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (client_id, machine_id)
        DO UPDATE SET
            server_id = $2,
            jurisdiction_id = $5,
            common_access_card_id = $6,
            given_name = $7,
            family_name = $8,
            created_at = $9
        "#,
        ClientId::new().as_uuid(),
        registration_request.server_id.as_uuid(),
        registration_request.client_id.as_uuid(),
        registration_request.machine_id,
        registration_request.jurisdiction_id.as_uuid(),
        registration_request.common_access_card_id,
        registration_request.given_name,
        registration_request.family_name,
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

pub(crate) async fn get_registration_requests_to_sync_to_cacvote_server(
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
            family_name
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
        })
        .collect())
}

pub(crate) async fn get_elections_to_sync_to_cacvote_server(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<client::input::Election>> {
    let records = sqlx::query!(
        r#"
        SELECT
            jurisdiction_id as "jurisdiction_id: ServerId",
            client_id as "client_id: ClientId",
            machine_id,
            definition,
            return_address
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
                jurisdiction_id: e.jurisdiction_id,
                client_id: e.client_id,
                machine_id: e.machine_id,
                definition: String::from_utf8(e.definition)?.parse()?,
                return_address: e.return_address,
            })
        })
        .collect()
}

pub(crate) async fn get_registrations_to_sync_to_cacvote_server(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<client::input::Registration>> {
    let records = sqlx::query!(
        r#"
        SELECT
            client_id as "client_id: ClientId",
            machine_id,
            jurisdiction_id as "jurisdiction_id: ServerId",
            (SELECT client_id FROM registration_requests WHERE id = registration_request_id) as "registration_request_id!: ClientId",
            (SELECT client_id FROM elections WHERE id = election_id) as "election_id!: ClientId",
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

    records
        .into_iter()
        .map(|r| {
            Ok(client::input::Registration {
                client_id: r.client_id,
                machine_id: r.machine_id,
                jurisdiction_id: r.jurisdiction_id,
                election_id: r.election_id,
                registration_request_id: r.registration_request_id,
                common_access_card_id: r.common_access_card_id,
                precinct_id: r.precinct_id,
                ballot_style_id: r.ballot_style_id,
            })
        })
        .collect()
}

pub(crate) async fn get_printed_ballots_to_sync_to_cacvote_server(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<client::input::PrintedBallot>> {
    let records = sqlx::query!(
        r#"
        SELECT
            client_id as "client_id: ClientId",
            machine_id,
            common_access_card_id,
            common_access_card_certificate,
            (SELECT client_id FROM registrations WHERE id = registration_id) as "registration_id!: ClientId",
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

    fn build_cacvote_server_jurisdiction() -> client::output::Jurisdiction {
        client::output::Jurisdiction {
            id: ServerId::new(),
            code: "st.test-jurisdiction".to_owned(),
            name: "Test Jurisdiction".to_owned(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    fn build_cacvote_server_registration_request(
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
            created_at: OffsetDateTime::now_utc(),
        }
    }

    fn build_cacvote_server_election(
        jurisdiction_id: ServerId,
        election_definition: ElectionDefinition,
    ) -> client::output::Election {
        client::output::Election {
            server_id: ServerId::new(),
            client_id: ClientId::new(),
            machine_id: "mark-terminal-001".to_owned(),
            jurisdiction_id,
            return_address: "123 Main St., Anytown, USA".to_owned(),
            election_hash: election_definition.election_hash.clone(),
            definition: election_definition,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    fn build_cacvote_server_registration(
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
            common_access_card_id: registration_request.common_access_card_id.clone(),
            registration_request_id: registration_request.server_id,
            election_id: election.server_id,
            precinct_id: ballot_style.precincts[0].to_string(),
            ballot_style_id: ballot_style.id.to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    fn build_cacvote_server_printed_ballot(
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

    fn build_config() -> Config {
        Config {
            cacvote_url: reqwest::Url::parse("http://localhost:8000").unwrap(),
            database_url: "postgres:test".to_owned(),
            machine_id: "cacvote-jx-test".to_owned(),
            port: 5000,
            public_dir: None,
            log_level: tracing::Level::INFO,
        }
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_add_election_from_cacvote_server(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let mut db = pool.acquire().await?;

        let election_definition = load_famous_names_election();
        let jurisdiction = build_cacvote_server_jurisdiction();
        let record = build_cacvote_server_election(jurisdiction.id, election_definition.clone());

        add_or_update_jurisdiction_from_cacvote_server(&mut db, jurisdiction)
            .await
            .unwrap();

        let client_id = add_election_from_cacvote_server(&mut db, record.clone())
            .await
            .unwrap();

        // insert again, should be idempotent
        let client_id2 = add_election_from_cacvote_server(&mut db, record.clone())
            .await
            .unwrap();

        assert_eq!(client_id, client_id2);

        Ok(())
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_add_election_to_be_synced(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let mut db = pool.acquire().await?;

        let election_definition = load_famous_names_election();
        let jurisdiction = build_cacvote_server_jurisdiction();

        let jurisdiction_id = add_or_update_jurisdiction_from_cacvote_server(&mut db, jurisdiction)
            .await
            .unwrap();

        let client_id = add_election(
            &mut db,
            &build_config(),
            jurisdiction_id,
            election_definition.clone(),
            "123 Main St.\nAnytown, USA",
        )
        .await
        .unwrap();

        let elections = get_elections_to_sync_to_cacvote_server(&mut db)
            .await
            .unwrap();

        assert_eq!(elections.len(), 1);
        assert_eq!(elections[0].client_id, client_id);
        assert_eq!(
            elections[0].definition.election_data,
            election_definition.election_data
        );

        Ok(())
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_add_everything_to_database(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let mut db = pool.acquire().await?;

        let election_definition = load_famous_names_election();
        let jurisdiction = build_cacvote_server_jurisdiction();
        let registration_request = build_cacvote_server_registration_request(jurisdiction.id);
        let election = build_cacvote_server_election(jurisdiction.id, election_definition.clone());
        let registration = build_cacvote_server_registration(
            &registration_request,
            &election,
            &election_definition,
        );
        let printed_ballot = build_cacvote_server_printed_ballot(&registration, Cvr::default());

        add_or_update_jurisdiction_from_cacvote_server(&mut db, jurisdiction)
            .await
            .unwrap();

        add_election_from_cacvote_server(&mut db, election)
            .await
            .unwrap();

        add_or_update_registration_request_from_cacvote_server(&mut db, registration_request)
            .await
            .unwrap();

        add_or_update_registration_from_cacvote_server(&mut db, registration)
            .await
            .unwrap();

        add_or_update_printed_ballot_from_cacvote_server(&mut db, printed_ballot)
            .await
            .unwrap();

        Ok(())
    }
}
