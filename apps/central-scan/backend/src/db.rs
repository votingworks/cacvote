extern crate time;

use rocket::{fairing, Build, Rocket};
use rocket_db_pools::{sqlx, Database};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, Postgres};
use types_rs::cdf::cvr::Cvr;
use types_rs::election::ElectionDefinition;
use types_rs::rave::{client, ClientId, ServerId};

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
    pub server_id: ServerId,
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

pub(crate) async fn add_admin_from_rave_server(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
    admin: client::output::Admin,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO admins (
            common_access_card_id
        )
        VALUES ($1)
        "#,
        admin.common_access_card_id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub(crate) async fn get_admins(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
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
    .fetch_all(&mut *db)
    .await
}

pub(crate) async fn get_last_synced_election_id(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
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
    .fetch_optional(&mut *db)
    .await?
    .map(|r| r.server_id))
}

pub(crate) async fn get_elections(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
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
        .fetch_optional(&mut *db)
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
                    id as "id: ClientId",
                    server_id as "server_id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    election as "election: String",
                    election_hash,
                    created_at
                FROM elections
                WHERE created_at > $1
                ORDER BY created_at DESC
                "#,
                election.created_at
            )
            .fetch_all(&mut *db)
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
                    election as "election: String",
                    election_hash,
                    created_at
                FROM elections
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&mut *db)
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

pub(crate) async fn add_election_from_rave_server(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
    record: client::output::Election,
) -> color_eyre::Result<ClientId> {
    let election_id = ClientId::new();

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
        DO NOTHING
        "#,
        election_id.as_uuid(),
        record.server_id.as_uuid(),
        record.client_id.as_uuid(),
        record.machine_id,
        record.election.election_hash,
        Json(record.election.election) as _
    )
    .execute(db)
    .await?;

    Ok(election_id)
}

pub(crate) async fn add_or_update_scanned_ballot_from_rave_server(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
    scanned_ballot: types_rs::rave::client::output::ScannedBallot,
) -> color_eyre::Result<ClientId> {
    let scanned_ballot_id = ClientId::new();

    sqlx::query!(
        r#"
        INSERT INTO scanned_ballots (
            id,
            server_id,
            client_id,
            machine_id,
            cast_vote_record
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (client_id, machine_id)
        DO UPDATE SET
            server_id = $2,
            cast_vote_record = $5
        "#,
        scanned_ballot_id.as_uuid(),
        scanned_ballot.server_id.as_uuid(),
        scanned_ballot.client_id.as_uuid(),
        scanned_ballot.machine_id,
        Json(scanned_ballot.cast_vote_record) as _
    )
    .execute(db)
    .await?;

    Ok(scanned_ballot_id)
}

pub(crate) async fn get_scanned_ballots_to_sync_to_rave_server(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
) -> color_eyre::Result<Vec<client::input::ScannedBallot>> {
    let records = sqlx::query!(
        r#"
        SELECT
            id as "id: ClientId",
            server_id as "server_id: ServerId",
            client_id as "client_id: ClientId",
            machine_id,
            (SELECT server_id FROM elections WHERE id = election_id) as "election_id!: ServerId",
            cast_vote_record as "cast_vote_record: String",
            created_at
        FROM scanned_ballots
        WHERE server_id IS NULL
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&mut *db)
    .await?;

    records
        .into_iter()
        .map(|b| {
            Ok(client::input::ScannedBallot {
                client_id: b.client_id,
                machine_id: b.machine_id,
                election_id: b.election_id,
                cast_vote_record: serde_json::from_str(b.cast_vote_record.as_str())?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) async fn add_scanned_ballot(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
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
    } = dbg!(scanned_ballot);
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
    .execute(db)
    .await?;

    Ok(())
}

pub(crate) async fn get_last_synced_scanned_ballot_id(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
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
    .fetch_optional(&mut *db)
    .await?
    .and_then(|r| r.server_id))
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScannedBallotStats {
    pub total: i64,
    pub pending: i64,
}

pub(crate) async fn get_scanned_ballot_stats(
    db: &mut ::sqlx::pool::PoolConnection<Postgres>,
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
    .fetch_one(&mut *db)
    .await
    .map_err(Into::into)
}
