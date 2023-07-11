extern crate time;

use rocket::{fairing, Build, Rocket};
use rocket_db_pools::{sqlx, Database};
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, types::Json, Postgres};
use uuid::Uuid;

#[derive(Database)]
#[database("sqlx")]
pub(crate) struct Db(sqlx::PgPool);

pub(crate) async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/migrations").run(&**db).await {
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
pub(crate) struct Election {
    pub id: Uuid,
    pub election: Json<crate::cvr::Election>,
    pub created_at: time::PrimitiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Voter {
    pub id: Uuid,
    pub common_access_card_id: String,
    pub is_admin: bool,
    pub created_at: time::PrimitiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct VoterRegistrationRequest {
    pub id: Uuid,
    pub voter_id: Uuid,
    pub given_name: String,
    pub family_name: String,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub state_id: String,
    pub created_at: time::PrimitiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct VoterElectionRegistration {
    pub id: Uuid,
    pub voter_id: Uuid,
    pub voter_registration_request_id: Uuid,
    pub election_id: Uuid,
    pub precinct_id: String,
    pub ballot_style_id: String,
    pub created_at: time::PrimitiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct VoterElectionSelection {
    pub id: Uuid,
    pub voter_id: Uuid,
    pub voter_election_registration_id: Uuid,
    pub cast_vote_record: Json<crate::cvr::CastVoteRecordReport>,
    pub created_at: time::PrimitiveDateTime,
}

pub(crate) async fn create_election(
    db: &mut PoolConnection<Postgres>,
    id: Uuid,
    election: crate::cvr::Election,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO elections (
            id,
            election
        )
        VALUES ($1, $2)
        "#,
        id,
        Json(election) as _
    )
    .execute(&mut *db)
    .await
    .map(|_| ())
}

pub(crate) async fn get_elections(
    db: &mut PoolConnection<Postgres>,
    since_election_id: Option<Uuid>,
) -> Result<Vec<Election>, sqlx::Error> {
    let since_election = match since_election_id {
        Some(id) => sqlx::query!(
            r#"
            SELECT created_at
            FROM elections
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&mut *db)
        .await
        .ok(),
        None => None,
    }
    .flatten();

    sqlx::query_as!(
        Election,
        r#"
        SELECT
            id,
            election as "election: _",
            created_at
        FROM elections
        WHERE created_at > $1
        ORDER BY created_at DESC
        "#,
        since_election.map_or_else(|| time::PrimitiveDateTime::MIN, |e| e.created_at)
    )
    .fetch_all(&mut *db)
    .await
}

pub(crate) async fn get_voter_election_registrations(
    db: &mut PoolConnection<Postgres>,
    since_voter_election_registration_id: Option<Uuid>,
) -> Result<Vec<VoterElectionRegistration>, sqlx::Error> {
    let since_voter_election_registration = sqlx::query!(
        r#"
        SELECT created_at
        FROM voter_election_registrations
        WHERE id = $1
        "#,
        since_voter_election_registration_id
    )
    .fetch_optional(&mut *db)
    .await
    .ok()
    .flatten();

    sqlx::query_as!(
        VoterElectionRegistration,
        r#"
        SELECT *
        FROM voter_election_registrations
        WHERE created_at > $1
        ORDER BY created_at DESC
        "#,
        since_voter_election_registration
            .map_or_else(|| time::PrimitiveDateTime::MIN, |e| e.created_at)
    )
    .fetch_all(&mut *db)
    .await
}

pub(crate) async fn get_voters(
    db: &mut PoolConnection<Postgres>,
    since: Option<time::PrimitiveDateTime>,
) -> Result<Vec<Voter>, sqlx::Error> {
    sqlx::query_as!(
        Voter,
        r#"
        SELECT *
        FROM voters
        WHERE created_at > $1
        ORDER BY created_at DESC
        "#,
        since.unwrap_or(time::PrimitiveDateTime::MIN)
    )
    .fetch_all(&mut *db)
    .await
}

pub(crate) async fn add_voter_registration_request(
    db: &mut PoolConnection<Postgres>,
    voter_registration_request: &VoterRegistrationRequest,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO voter_registration_requests (
            id,
            voter_id,
            given_name,
            family_name,
            address_line_1,
            address_line_2,
            city,
            state,
            postal_code,
            state_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (id) DO NOTHING
        "#,
        voter_registration_request.id,
        voter_registration_request.voter_id,
        voter_registration_request.given_name,
        voter_registration_request.family_name,
        voter_registration_request.address_line_1,
        voter_registration_request.address_line_2,
        voter_registration_request.city,
        voter_registration_request.state,
        voter_registration_request.postal_code,
        voter_registration_request.state_id
    )
    .execute(db)
    .await
    .map(|_| ())
}

pub(crate) async fn add_voter_election_selection(
    db: &mut PoolConnection<Postgres>,
    voter_election_selection: &VoterElectionSelection,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO voter_election_selections (
            id,
            voter_id,
            voter_election_registration_id,
            cast_vote_record
        )
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO NOTHING
        "#,
        voter_election_selection.id,
        voter_election_selection.voter_id,
        voter_election_selection.voter_election_registration_id,
        voter_election_selection.cast_vote_record as _
    )
    .execute(&mut *db)
    .await
    .map(|_| ())
}

pub(crate) async fn get_voter_election_selections(
    db: &mut PoolConnection<Postgres>,
    since_voter_election_selection_id: Option<Uuid>,
) -> Result<Vec<VoterElectionSelection>, sqlx::Error> {
    let since_voter_election_selection = match since_voter_election_selection_id {
        Some(id) => sqlx::query!(
            r#"
            SELECT created_at
            FROM voter_election_selections
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&mut *db)
        .await
        .ok()
        .flatten(),
        None => None,
    };

    sqlx::query_as!(
        VoterElectionSelection,
        r#"
        SELECT
            id,
            voter_id,
            voter_election_registration_id,
            cast_vote_record as "cast_vote_record: _",
            created_at
        FROM voter_election_selections
        WHERE created_at > $1
        ORDER BY created_at DESC
        "#,
        since_voter_election_selection
            .map_or_else(|| time::PrimitiveDateTime::MIN, |e| e.created_at)
    )
    .fetch_all(&mut *db)
    .await
}
