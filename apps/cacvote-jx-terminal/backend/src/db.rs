//! Database access for the application.
//!
//! All direct use of [SQLx][`sqlx`] queries should be in this module. When
//! modifying this file, be sure to run `cargo sqlx prepare --workspace` in the
//! workspace root to regenerate the query metadata for offline builds.
//!
//! To enable `cargo sqlx prepare --workspace`, install it via `cargo install
//! --locked sqlx-cli`.

use std::time::Duration;

use base64_serde::base64_serde_type;
use color_eyre::eyre::bail;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgPool};
use tracing::Level;
use types_rs::cacvote;
use uuid::Uuid;

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

pub(crate) async fn get_elections(
    connection: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Vec<cacvote::ElectionPresenter>> {
    let objects = sqlx::query_as!(
        cacvote::SignedObject,
        r#"
        SELECT
            id,
            election_id,
            payload,
            certificate,
            signature
        FROM objects
        WHERE object_type = 'Election'
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&mut *connection)
    .await?;

    let mut elections = Vec::new();

    for object in objects {
        let payload = match object.try_to_inner() {
            Ok(payload) => {
                tracing::trace!("got object payload: {payload:?}");
                payload
            }
            Err(err) => {
                tracing::error!("unable to parse object payload: {err:?}");
                continue;
            }
        };

        // FIXME: this makes `get_elections` run N+1 queries
        let (encrypted_tally, decrypted_tally) =
            get_tallies_for_election(&mut *connection, object.id)
                .await?
                .into();

        let shuffled_encrypted_cast_ballots =
            get_shuffled_encrypted_cast_ballots(&mut *connection, &object.id).await?;

        if let cacvote::Payload::Election(election) = payload {
            elections.push(cacvote::ElectionPresenter::new(
                object.id,
                election,
                encrypted_tally,
                decrypted_tally,
                shuffled_encrypted_cast_ballots,
            ));
        }
    }

    Ok(elections)
}

#[tracing::instrument(skip(connection))]
pub async fn get_pending_registration_requests(
    connection: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Vec<cacvote::RegistrationRequestPresenter>> {
    let records = sqlx::query!(
        r#"
        SELECT
            rr.id,
            rr.election_id,
            rr.payload,
            rr.certificate,
            rr.signature,
            rr.created_at
        FROM
            objects AS rr
        WHERE
            rr.object_type = $1
          AND
            NOT EXISTS (
                SELECT 1
                FROM objects AS r
                WHERE r.object_type = $2
                  AND rr.id = (convert_from(r.payload, 'UTF8')::jsonb ->> $3)::uuid
            )
        ORDER BY rr.created_at DESC
        "#,
        cacvote::Payload::registration_request_object_type(),
        cacvote::Payload::registration_object_type(),
        cacvote::Registration::registration_request_object_id_field_name(),
    )
    .fetch_all(connection)
    .await?;

    let mut registration_requests = Vec::new();

    for record in records {
        let object = cacvote::SignedObject {
            id: record.id,
            election_id: record.election_id,
            payload: record.payload,
            certificate: record.certificate,
            signature: record.signature,
        };

        if let cacvote::Payload::RegistrationRequest(registration_request) =
            object.try_to_inner()?
        {
            registration_requests.push(cacvote::RegistrationRequestPresenter::new(
                object.id,
                format!(
                    "{} {}",
                    registration_request.given_name, registration_request.family_name
                ),
                registration_request,
                record.created_at,
            ));
        }
    }

    Ok(registration_requests)
}

#[tracing::instrument(skip(connection))]
pub async fn get_registrations(
    connection: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Vec<cacvote::RegistrationPresenter>> {
    let records = sqlx::query!(
        r#"
        SELECT
            r.id AS registration_id,
            r.payload AS registration_payload,
            r.certificate AS registration_certificate,
            r.signature AS registration_signature,
            e.id AS election_id,
            e.election_id AS election_election_id,
            e.payload AS election_payload,
            e.certificate AS election_certificate,
            e.signature AS election_signature,
            rr.id AS registration_request_id,
            rr.election_id AS registration_request_election_id,
            rr.payload AS registration_request_payload,
            rr.certificate AS registration_request_certificate,
            rr.signature AS registration_request_signature,
            r.created_at AS created_at,
            r.server_synced_at IS NOT NULL AS "is_synced!: bool"
        FROM objects AS r
        INNER JOIN objects AS e
            ON (convert_from(r.payload, 'UTF8')::jsonb ->> $1)::uuid = e.id
        INNER JOIN objects AS rr
            ON (convert_from(r.payload, 'UTF8')::jsonb ->> $2)::uuid = rr.id
        WHERE e.object_type = $3
          AND r.object_type = $4
        ORDER BY r.created_at DESC
        "#,
        cacvote::Registration::election_object_id_field_name(),
        cacvote::Registration::registration_request_object_id_field_name(),
        cacvote::Payload::election_object_type(),
        cacvote::Payload::registration_object_type(),
    )
    .fetch_all(connection)
    .await?;

    let mut registrations = Vec::new();

    for record in records {
        let registration_object = cacvote::SignedObject {
            id: record.registration_id,
            election_id: Some(record.election_id),
            payload: record.registration_payload,
            certificate: record.registration_certificate,
            signature: record.registration_signature,
        };
        let election_object = cacvote::SignedObject {
            id: record.election_id,
            election_id: record.election_election_id,
            payload: record.election_payload,
            certificate: record.election_certificate,
            signature: record.election_signature,
        };
        let registration_request_object = cacvote::SignedObject {
            id: record.registration_request_id,
            election_id: record.registration_request_election_id,
            payload: record.registration_request_payload,
            certificate: record.registration_request_certificate,
            signature: record.registration_request_signature,
        };

        if let cacvote::Payload::Registration(registration) = registration_object.try_to_inner()? {
            if let cacvote::Payload::Election(election_payload) = election_object.try_to_inner()? {
                if let cacvote::Payload::RegistrationRequest(registration_request) =
                    registration_request_object.try_to_inner()?
                {
                    let display_name = registration_request.display_name();
                    let election_title = election_payload.election.title.clone();
                    let election_hash = election_payload.election_hash.clone();
                    let created_at = record.created_at;
                    let is_synced = record.is_synced;
                    registrations.push(cacvote::RegistrationPresenter::new(
                        registration_object.id,
                        display_name,
                        election_title,
                        election_hash,
                        registration,
                        created_at,
                        is_synced,
                    ));
                }
            }
        }
    }

    Ok(registrations)
}

#[tracing::instrument(skip(connection))]
pub async fn get_registration_request(
    connection: &mut sqlx::PgConnection,
    id: Uuid,
) -> color_eyre::Result<cacvote::RegistrationRequest> {
    if let cacvote::Payload::RegistrationRequest(registration_request) =
        get_object(connection, id).await?.try_to_inner()?
    {
        Ok(registration_request)
    } else {
        bail!("Object is not a registration request")
    }
}

#[tracing::instrument(skip(connection))]
pub async fn get_object(
    connection: &mut sqlx::PgConnection,
    id: Uuid,
) -> color_eyre::Result<cacvote::SignedObject> {
    let object = sqlx::query_as!(
        cacvote::SignedObject,
        r#"
        SELECT
            id,
            election_id,
            payload,
            certificate,
            signature
        FROM objects
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(connection)
    .await?;

    // Ensure the denormalized election_id field matches the election_id in the
    // payload. The denormalized field is used for fast lookups, but isn't part
    // of the signed payload.
    assert_eq!(object.election_id, object.try_to_inner()?.election_id());

    Ok(object)
}

#[derive(Debug)]
pub enum ElectionTallies {
    Neither,
    OnlyEncrypted(cacvote::EncryptedElectionTallyPresenter),
    Both(
        cacvote::EncryptedElectionTallyPresenter,
        cacvote::DecryptedElectionTallyPresenter,
    ),
}

impl From<ElectionTallies>
    for (
        Option<cacvote::EncryptedElectionTallyPresenter>,
        Option<cacvote::DecryptedElectionTallyPresenter>,
    )
{
    fn from(tallies: ElectionTallies) -> Self {
        match tallies {
            ElectionTallies::Neither => (None, None),
            ElectionTallies::OnlyEncrypted(encrypted_tally) => (Some(encrypted_tally), None),
            ElectionTallies::Both(encrypted_tally, decrypted_tally) => {
                (Some(encrypted_tally), Some(decrypted_tally))
            }
        }
    }
}

#[tracing::instrument(skip(connection))]
pub async fn get_tallies_for_election(
    connection: &mut sqlx::PgConnection,
    election_object_id: Uuid,
) -> color_eyre::Result<ElectionTallies> {
    let Some(record) = sqlx::query!(
        r#"
        SELECT
            payload,
            created_at,
            server_synced_at
        FROM objects
        WHERE object_type = $1
          AND (convert_from(payload, 'UTF8')::jsonb ->> $2)::uuid = $3
        "#,
        cacvote::Payload::encrypted_election_tally_object_type(),
        cacvote::EncryptedElectionTally::election_object_id_field_name(),
        election_object_id,
    )
    .fetch_optional(&mut *connection)
    .await?
    else {
        return Ok(ElectionTallies::Neither);
    };

    let cacvote::Payload::EncryptedElectionTally(encrypted_election_tally) =
        serde_json::from_slice(&record.payload)?
    else {
        bail!("Object is not an encrypted election tally")
    };

    let encrypted_election_tally = cacvote::EncryptedElectionTallyPresenter {
        encrypted_election_tally,
        created_at: record.created_at,
        synced_at: record.server_synced_at,
    };

    let Some(record) = sqlx::query!(
        r#"
        SELECT
            payload,
            created_at,
            server_synced_at
        FROM objects
        WHERE object_type = $1
          AND (convert_from(payload, 'UTF8')::jsonb ->> $2)::uuid = $3
        "#,
        cacvote::Payload::decrypted_election_tally_object_type(),
        cacvote::DecryptedElectionTally::election_object_id_field_name(),
        election_object_id,
    )
    .fetch_optional(&mut *connection)
    .await?
    else {
        return Ok(ElectionTallies::OnlyEncrypted(encrypted_election_tally));
    };

    let cacvote::Payload::DecryptedElectionTally(decrypted_election_tally) =
        serde_json::from_slice(&record.payload)?
    else {
        bail!("Object is not a decrypted election tally")
    };

    let decrypted_election_tally = cacvote::DecryptedElectionTallyPresenter {
        decrypted_election_tally,
        created_at: record.created_at,
        synced_at: record.server_synced_at,
    };

    Ok(ElectionTallies::Both(
        encrypted_election_tally,
        decrypted_election_tally,
    ))
}

/// Adds an object to the database from the CACvote server. The object should
/// already have had its certificate and signature verified.
#[tracing::instrument(skip(connection, object))]
pub async fn add_object_from_server(
    connection: &mut sqlx::PgConnection,
    object: &cacvote::SignedObject,
) -> color_eyre::Result<Uuid> {
    let Some(jurisdiction_code) = object.jurisdiction_code() else {
        bail!("No jurisdiction found");
    };

    let object_type = object.try_to_inner()?.object_type();

    sqlx::query!(
        r#"
        INSERT INTO objects (id, election_id, jurisdiction, object_type, payload, certificate, signature, server_synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, now())
        "#,
        &object.id,
        object.election_id,
        jurisdiction_code.as_str(),
        object_type,
        &object.payload,
        &object.certificate,
        &object.signature
    )
    .execute(connection)
    .await?;

    tracing::debug!("Created object with id {}", object.id);

    Ok(object.id)
}

/// Adds an object to the database. The object should already have had its
/// certificate and signature verified.
#[tracing::instrument(skip(connection, object))]
pub async fn add_object(
    connection: &mut sqlx::PgConnection,
    object: &cacvote::SignedObject,
) -> color_eyre::Result<Uuid> {
    let Some(jurisdiction_code) = object.jurisdiction_code() else {
        bail!("No jurisdiction found");
    };

    let object_type = object.try_to_inner()?.object_type();

    sqlx::query!(
        r#"
        INSERT INTO objects (id, election_id, jurisdiction, object_type, payload, certificate, signature)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        &object.id,
        object.election_id,
        jurisdiction_code.as_str(),
        object_type,
        &object.payload,
        &object.certificate,
        &object.signature
    )
    .execute(connection)
    .await?;

    tracing::info!("Created object with id {}", object.id);

    Ok(object.id)
}

#[tracing::instrument(skip(connection, entries))]
pub(crate) async fn add_journal_entries(
    connection: &mut sqlx::PgConnection,
    entries: Vec<cacvote::JournalEntry>,
) -> color_eyre::eyre::Result<()> {
    let mut txn = connection.begin().await?;
    for entry in entries {
        sqlx::query!(
            r#"
            INSERT INTO journal_entries (id, object_id, jurisdiction, object_type, action, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO NOTHING
            "#,
            entry.id,
            entry.object_id,
            entry.jurisdiction_code.as_str(),
            entry.object_type,
            entry.action.as_str(),
            entry.created_at
        )
        .execute(&mut *txn)
        .await?;
    }
    txn.commit().await?;
    Ok(())
}

pub(crate) async fn get_latest_journal_entry(
    connection: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Option<cacvote::JournalEntry>> {
    Ok(sqlx::query_as!(
        cacvote::JournalEntry,
        r#"
        SELECT
            id,
            object_id,
            election_id,
            jurisdiction as "jurisdiction_code: cacvote::JurisdictionCode",
            object_type,
            action,
            created_at
        FROM journal_entries
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut *connection)
    .await?)
}

pub(crate) async fn get_unsynced_objects(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Vec<cacvote::SignedObject>> {
    Ok(sqlx::query_as!(
        cacvote::SignedObject,
        r#"
        SELECT
            id,
            election_id,
            payload,
            certificate,
            signature
        FROM objects
        WHERE server_synced_at IS NULL
        "#,
    )
    .fetch_all(&mut *executor)
    .await?)
}

pub(crate) async fn mark_object_synced(
    executor: &mut sqlx::PgConnection,
    id: uuid::Uuid,
) -> color_eyre::eyre::Result<()> {
    sqlx::query!(
        r#"
        UPDATE objects
        SET server_synced_at = now()
        WHERE id = $1
        "#,
        id
    )
    .execute(&mut *executor)
    .await?;

    Ok(())
}

pub(crate) async fn get_journal_entries_for_objects_to_pull(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::eyre::Result<Vec<cacvote::JournalEntry>> {
    Ok(sqlx::query_as!(
        cacvote::JournalEntry,
        r#"
        SELECT
            id,
            object_id,
            election_id,
            jurisdiction as "jurisdiction_code: cacvote::JurisdictionCode",
            object_type,
            action,
            created_at
        FROM journal_entries
        WHERE object_id IS NOT NULL
          AND object_type IN ($1, $2)
          AND object_id NOT IN (SELECT id FROM objects)
        "#,
        cacvote::Payload::registration_request_object_type(),
        cacvote::Payload::cast_ballot_object_type(),
    )
    .fetch_all(&mut *executor)
    .await?)
}

pub(crate) async fn get_cast_ballots(
    executor: &mut sqlx::PgConnection,
) -> color_eyre::Result<Vec<cacvote::CastBallotPresenter>> {
    let records = sqlx::query!(
        r#"
        SELECT
            cb.id AS cast_ballot_id,
            cb.election_id as cast_ballot_election_id,
            cb.payload AS cast_ballot_payload,
            cb.certificate AS cast_ballot_certificate,
            cb.signature AS cast_ballot_signature,
            rr.id AS registration_request_id,
            rr.election_id AS registration_request_election_id,
            rr.payload AS registration_request_payload,
            rr.certificate AS registration_request_certificate,
            rr.signature AS registration_request_signature,
            r.id AS registration_id,
            r.election_id AS registration_election_id,
            r.payload AS registration_payload,
            r.certificate AS registration_certificate,
            r.signature AS registration_signature,
            cb.created_at AS created_at
        FROM objects AS cb
        -- join on registration request
        INNER JOIN objects AS rr
            ON (convert_from(cb.payload, 'UTF8')::jsonb ->> $1)::uuid = rr.id
        -- join on registration
        INNER JOIN objects AS r
            ON (convert_from(cb.payload, 'UTF8')::jsonb ->> $2)::uuid = r.id
        WHERE rr.object_type = $3
          AND cb.object_type = $4
          AND r.object_type = $5
        ORDER BY cb.created_at DESC
        "#,
        cacvote::CastBallot::registration_request_object_id_field_name(),
        cacvote::CastBallot::registration_object_id_field_name(),
        cacvote::Payload::registration_request_object_type(),
        cacvote::Payload::cast_ballot_object_type(),
        cacvote::Payload::registration_object_type(),
    )
    .fetch_all(&mut *executor)
    .await?;

    let mut cast_ballots = Vec::new();

    for record in records {
        let cast_ballot_object = cacvote::SignedObject {
            id: record.cast_ballot_id,
            election_id: record.cast_ballot_election_id,
            payload: record.cast_ballot_payload,
            certificate: record.cast_ballot_certificate,
            signature: record.cast_ballot_signature,
        };
        let registration_object = cacvote::SignedObject {
            id: record.registration_id,
            election_id: record.registration_election_id,
            payload: record.registration_payload,
            certificate: record.registration_certificate,
            signature: record.registration_signature,
        };
        let registration_request_object = cacvote::SignedObject {
            id: record.registration_request_id,
            election_id: record.registration_request_election_id,
            payload: record.registration_request_payload,
            certificate: record.registration_request_certificate,
            signature: record.registration_request_signature,
        };

        if let cacvote::Payload::CastBallot(cast_ballot) = cast_ballot_object.try_to_inner()? {
            if let cacvote::Payload::RegistrationRequest(registration_request) =
                registration_request_object.try_to_inner()?
            {
                if let cacvote::Payload::Registration(registration) =
                    registration_object.try_to_inner()?
                {
                    // TODO: remove this or replace with actual verification status
                    // we already verify the signature as part of adding the object to the DB,
                    // so we can assume that the verification status is success
                    let verification_status = cacvote::VerificationStatus::Success {
                        common_access_card_id: cast_ballot.common_access_card_id.clone(),
                        display_name: "test".to_owned(),
                    };
                    let created_at = record.created_at;
                    cast_ballots.push(cacvote::CastBallotPresenter::new(
                        cast_ballot,
                        registration_request,
                        registration,
                        registration_object.id,
                        verification_status,
                        created_at,
                    ));
                }
            }
        }
    }

    Ok(cast_ballots)
}

pub(crate) async fn get_cast_ballots_for_election(
    executor: &mut sqlx::PgConnection,
    election_object_id: &Uuid,
) -> color_eyre::Result<Vec<cacvote::CastBallot>> {
    let records = sqlx::query!(
        r#"
        SELECT
            cb.id AS cast_ballot_id,
            cb.election_id as cast_ballot_election_id,
            cb.payload AS cast_ballot_payload,
            cb.certificate AS cast_ballot_certificate,
            cb.signature AS cast_ballot_signature
        FROM objects AS cb
        WHERE cb.object_type = $1
          AND (convert_from(cb.payload, 'UTF8')::jsonb ->> $2)::uuid = $3
        "#,
        cacvote::Payload::cast_ballot_object_type(),
        cacvote::CastBallot::election_object_id_field_name(),
        election_object_id,
    )
    .fetch_all(executor)
    .await?;

    let mut cast_ballots = Vec::new();

    for record in records {
        let cast_ballot = cacvote::SignedObject {
            id: record.cast_ballot_id,
            election_id: record.cast_ballot_election_id,
            payload: record.cast_ballot_payload,
            certificate: record.cast_ballot_certificate,
            signature: record.cast_ballot_signature,
        };

        if let cacvote::Payload::CastBallot(cast_ballot) = cast_ballot.try_to_inner()? {
            cast_ballots.push(cast_ballot);
        }
    }

    Ok(cast_ballots)
}

pub(crate) async fn get_shuffled_encrypted_cast_ballots(
    executor: &mut sqlx::PgConnection,
    election_object_id: &Uuid,
) -> color_eyre::Result<Option<cacvote::ShuffledEncryptedCastBallotsPresenter>> {
    let Some(record) = sqlx::query!(
        r#"
        SELECT
            b.id AS shuffled_encrypted_cast_ballots_id,
            b.election_id AS shuffled_encrypted_cast_ballots_election_id,
            b.payload AS shuffled_encrypted_cast_ballots_payload,
            b.certificate AS shuffled_encrypted_cast_ballots_certificate,
            b.signature AS shuffled_encrypted_cast_ballots_signature,
            b.created_at AS shuffled_encrypted_cast_ballots_created_at,
            b.server_synced_at AS shuffled_encrypted_cast_ballots_server_synced_at
        FROM objects AS b
        WHERE b.object_type = $1
          AND (convert_from(b.payload, 'UTF8')::jsonb ->> $2)::uuid = $3
        "#,
        cacvote::Payload::shuffled_encrypted_cast_ballots_object_type(),
        cacvote::ShuffledEncryptedCastBallots::election_object_id_field_name(),
        election_object_id,
    )
    .fetch_optional(executor)
    .await?
    else {
        return Ok(None);
    };

    let shuffled_encrypted_cast_ballots = cacvote::SignedObject {
        id: record.shuffled_encrypted_cast_ballots_id,
        election_id: record.shuffled_encrypted_cast_ballots_election_id,
        payload: record.shuffled_encrypted_cast_ballots_payload,
        certificate: record.shuffled_encrypted_cast_ballots_certificate,
        signature: record.shuffled_encrypted_cast_ballots_signature,
    };

    let cacvote::Payload::ShuffledEncryptedCastBallots(shuffled_encrypted_cast_ballots) =
        shuffled_encrypted_cast_ballots.try_to_inner()?
    else {
        bail!("Object is not a shuffled encrypted cast ballots")
    };

    let shuffled_encrypted_cast_ballots = cacvote::ShuffledEncryptedCastBallotsPresenter {
        shuffled_encrypted_cast_ballots,
        created_at: record.shuffled_encrypted_cast_ballots_created_at,
        synced_at: record.shuffled_encrypted_cast_ballots_server_synced_at,
    };

    Ok(Some(shuffled_encrypted_cast_ballots))
}

pub(crate) async fn add_eg_private_key(
    executor: &mut sqlx::PgConnection,
    election_object_id: &Uuid,
    private_metadata_blob: &[u8],
) -> color_eyre::Result<Uuid> {
    let record = sqlx::query!(
        r#"
        INSERT INTO eg_private_keys (election_object_id, private_key)
        VALUES ($1, $2)
        RETURNING id
        "#,
        election_object_id,
        private_metadata_blob
    )
    .fetch_one(executor)
    .await?;

    Ok(record.id)
}

pub(crate) async fn get_eg_private_key(
    executor: &mut sqlx::PgConnection,
    election_object_id: &Uuid,
) -> color_eyre::Result<Vec<u8>> {
    let record = sqlx::query!(
        r#"
        SELECT private_key
        FROM eg_private_keys
        WHERE election_object_id = $1
        "#,
        election_object_id
    )
    .fetch_one(executor)
    .await?;

    Ok(record.private_key)
}

#[cfg(test)]
mod tests {
    use openssl::{
        pkey::{PKey, Private, Public},
        x509::X509,
    };
    use types_rs::{cacvote::JurisdictionCode, election::ElectionDefinition};

    use super::*;

    fn load_keypair() -> color_eyre::Result<(X509, PKey<Public>, PKey<Private>)> {
        // uses the dev VxAdmin keypair because it has the Jurisdiction field
        let private_key_pem =
            include_bytes!("../../../../libs/auth/certs/dev/vx-admin-private-key.pem");
        let private_key = PKey::private_key_from_pem(private_key_pem)?;
        let certificate =
            include_bytes!("../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem")
                .to_vec();
        let x509 = X509::from_pem(&certificate)?;
        let public_key = x509.public_key()?;
        Ok((x509, public_key, private_key))
    }

    fn load_election_definition() -> color_eyre::Result<ElectionDefinition> {
        Ok(ElectionDefinition::try_from(
            &include_bytes!("../tests/fixtures/electionFamousNames2021.json")[..],
        )?)
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_pending_registration_requests(pool: sqlx::PgPool) -> color_eyre::Result<()> {
        let (certificate, _, private_key) = load_keypair()?;
        let election_definition = load_election_definition()?;
        let connection = &mut pool.acquire().await?;
        let jurisdiction_code = JurisdictionCode::try_from("st.test-jurisdiction").unwrap();

        let election_payload = cacvote::Payload::Election(cacvote::Election {
            jurisdiction_code: jurisdiction_code.clone(),
            election_definition: election_definition.clone(),
            mailing_address: "123 Main St".to_owned(),
            electionguard_election_metadata_blob: vec![],
        });
        let election_object = cacvote::SignedObject::from_payload(
            &election_payload,
            certificate.clone(),
            &private_key,
        )?;

        add_object_from_server(connection, &election_object).await?;

        let pending_registration_requests = get_pending_registration_requests(connection).await?;

        assert!(
            pending_registration_requests.is_empty(),
            "Expected no pending registration requests, got {pending_registration_requests:?}",
        );

        let registration_request_payload =
            cacvote::Payload::RegistrationRequest(cacvote::RegistrationRequest {
                jurisdiction_code: jurisdiction_code.clone(),
                common_access_card_id: "0123456789".to_owned(),
                family_name: "Smith".to_owned(),
                given_name: "John".to_owned(),
            });
        let registration_request_object = cacvote::SignedObject::from_payload(
            &registration_request_payload,
            certificate.clone(),
            &private_key,
        )?;

        add_object_from_server(connection, &registration_request_object).await?;

        let pending_registration_requests = get_pending_registration_requests(connection).await?;

        match pending_registration_requests.as_slice() {
            [registration_request] => {
                assert_eq!(registration_request.common_access_card_id, "0123456789");
                assert_eq!(registration_request.given_name, "John");
                assert_eq!(registration_request.family_name, "Smith");
                assert_eq!(registration_request.jurisdiction_code, jurisdiction_code);
            }
            _ => panic!("Expected one registration request, got {pending_registration_requests:?}"),
        }

        let ballot_style_id = election_definition.election.ballot_styles[0].id.clone();
        let precinct_id = election_definition.election.precincts[0].id.clone();

        let registration_payload = cacvote::Payload::Registration(cacvote::Registration {
            jurisdiction_code: jurisdiction_code.clone(),
            common_access_card_id: "0123456789".to_owned(),
            registration_request_object_id: registration_request_object.id,
            election_object_id: election_object.id,
            ballot_style_id,
            precinct_id,
        });
        let registration_object = cacvote::SignedObject::from_payload(
            &registration_payload,
            certificate.clone(),
            &private_key,
        )?;

        add_object_from_server(connection, &registration_object).await?;

        let pending_registration_requests = get_pending_registration_requests(connection).await?;

        assert!(
            pending_registration_requests.is_empty(),
            "Expected no pending registration requests, got {pending_registration_requests:?}",
        );

        Ok(())
    }
}
