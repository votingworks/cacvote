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
            payload,
            certificates,
            signature
        FROM objects
        WHERE object_type = 'Election'
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(connection)
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

        if let cacvote::Payload::Election(election) = payload {
            elections.push(cacvote::ElectionPresenter::new(object.id, election));
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
            rr.payload,
            rr.certificates,
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
            payload: record.payload,
            certificates: record.certificates,
            signature: record.signature,
        };

        if let cacvote::Payload::RegistrationRequest(registration_request) =
            object.try_to_inner()?
        {
            registration_requests.push(cacvote::RegistrationRequestPresenter::new(
                object.id,
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
            r.certificates AS registration_certificates,
            r.signature AS registration_signature,
            e.id AS election_id,
            e.payload AS election_payload,
            e.certificates AS election_certificates,
            e.signature AS election_signature,
            rr.id AS registration_request_id,
            rr.payload AS registration_request_payload,
            rr.certificates AS registration_request_certificates,
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
            payload: record.registration_payload,
            certificates: record.registration_certificates,
            signature: record.registration_signature,
        };
        let election_object = cacvote::SignedObject {
            id: record.election_id,
            payload: record.election_payload,
            certificates: record.election_certificates,
            signature: record.election_signature,
        };
        let registration_request_object = cacvote::SignedObject {
            id: record.registration_request_id,
            payload: record.registration_request_payload,
            certificates: record.registration_request_certificates,
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
    Ok(sqlx::query_as!(
        cacvote::SignedObject,
        r#"
        SELECT
            id,
            payload,
            certificates,
            signature
        FROM objects
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(connection)
    .await?)
}

#[tracing::instrument(skip(connection, object))]
pub async fn add_object_from_server(
    connection: &mut sqlx::PgConnection,
    object: &cacvote::SignedObject,
) -> color_eyre::Result<Uuid> {
    if !object.verify()? {
        bail!("Unable to verify signature/certificates")
    }

    let Some(jurisdiction_code) = object.jurisdiction_code() else {
        bail!("No jurisdiction found");
    };

    let object_type = object.try_to_inner()?.object_type();

    sqlx::query!(
        r#"
        INSERT INTO objects (id, jurisdiction, object_type, payload, certificates, signature, server_synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, now())
        "#,
        &object.id,
        jurisdiction_code.as_str(),
        object_type,
        &object.payload,
        &object.certificates,
        &object.signature
    )
    .execute(connection)
    .await?;

    tracing::debug!("Created object with id {}", object.id);

    Ok(object.id)
}

#[tracing::instrument(skip(connection, object))]
pub async fn add_object(
    connection: &mut sqlx::PgConnection,
    object: &cacvote::SignedObject,
) -> color_eyre::Result<Uuid> {
    if !object.verify()? {
        bail!("Unable to verify signature/certificates")
    }

    let Some(jurisdiction_code) = object.jurisdiction_code() else {
        bail!("No jurisdiction found");
    };

    let object_type = object.try_to_inner()?.object_type();

    sqlx::query!(
        r#"
        INSERT INTO objects (id, jurisdiction, object_type, payload, certificates, signature)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        &object.id,
        jurisdiction_code.as_str(),
        object_type,
        &object.payload,
        &object.certificates,
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
            payload,
            certificates,
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
            cb.payload AS cast_ballot_payload,
            cb.certificates AS cast_ballot_certificates,
            cb.signature AS cast_ballot_signature,
            rr.id AS registration_request_id,
            rr.payload AS registration_request_payload,
            rr.certificates AS registration_request_certificates,
            rr.signature AS registration_request_signature,
            r.id AS registration_id,
            r.payload AS registration_payload,
            r.certificates AS registration_certificates,
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
            payload: record.cast_ballot_payload,
            certificates: record.cast_ballot_certificates,
            signature: record.cast_ballot_signature,
        };
        let registration_object = cacvote::SignedObject {
            id: record.registration_id,
            payload: record.registration_payload,
            certificates: record.registration_certificates,
            signature: record.registration_signature,
        };
        let registration_request_object = cacvote::SignedObject {
            id: record.registration_request_id,
            payload: record.registration_request_payload,
            certificates: record.registration_request_certificates,
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
                        verification_status,
                        created_at,
                    ));
                }
            }
        }
    }

    Ok(cast_ballots)
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
        let certificates =
            include_bytes!("../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem")
                .to_vec();
        let x509 = X509::from_pem(&certificates)?;
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
        let (certificates, _, private_key) = load_keypair()?;
        let election_definition = load_election_definition()?;
        let mut connection = &mut pool.acquire().await?;
        let jurisdiction_code = JurisdictionCode::try_from("st.test-jurisdiction").unwrap();

        let election_payload = cacvote::Payload::Election(cacvote::Election {
            jurisdiction_code: jurisdiction_code.clone(),
            election_definition: election_definition.clone(),
            mailing_address: "123 Main St".to_owned(),
        });
        let election_object = cacvote::SignedObject::from_payload(
            &election_payload,
            vec![certificates.clone()],
            &private_key,
        )?;

        add_object_from_server(&mut connection, &election_object).await?;

        let pending_registration_requests =
            get_pending_registration_requests(&mut connection).await?;

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
            vec![certificates.clone()],
            &private_key,
        )?;

        add_object_from_server(&mut connection, &registration_request_object).await?;

        let pending_registration_requests =
            get_pending_registration_requests(&mut connection).await?;

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
            vec![certificates.clone()],
            &private_key,
        )?;

        add_object_from_server(&mut connection, &registration_object).await?;

        let pending_registration_requests =
            get_pending_registration_requests(&mut connection).await?;

        assert!(
            pending_registration_requests.is_empty(),
            "Expected no pending registration requests, got {pending_registration_requests:?}",
        );

        Ok(())
    }
}
