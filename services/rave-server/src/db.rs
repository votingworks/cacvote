extern crate time;

use rocket::{fairing, Build, Rocket};
use rocket_db_pools::{sqlx, Database};
use serde::{Deserialize, Serialize};
use sqlx::{pool::PoolConnection, types::Json, Postgres};
use uuid::Uuid;

use crate::client::{self, ClientId, ServerId};

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
    pub id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub election: Json<serde_json::Value>,
    pub created_at: sqlx::types::time::OffsetDateTime,
}

impl From<Election> for client::output::Election {
    fn from(election: Election) -> Self {
        let Election {
            id,
            client_id,
            machine_id,
            election,
            created_at,
        } = election;

        Self {
            server_id: id,
            client_id,
            machine_id,
            election,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RegistrationRequest {
    pub id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub common_access_card_id: String,
    pub given_name: String,
    pub family_name: String,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub state_id: String,
    pub created_at: sqlx::types::time::OffsetDateTime,
}

impl From<client::input::RegistrationRequest> for RegistrationRequest {
    fn from(request: client::input::RegistrationRequest) -> Self {
        let client::input::RegistrationRequest {
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
        } = request;

        Self {
            id: ServerId::new(),
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
            common_access_card_id,
            given_name,
            family_name,
            address_line_1,
            address_line_2,
            city,
            state,
            postal_code,
            state_id,
            created_at,
        } = request;

        Self {
            server_id: id,
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
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Registration {
    pub id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub common_access_card_id: String,
    pub registration_request_id: ServerId,
    pub election_id: ServerId,
    pub precinct_id: String,
    pub ballot_style_id: String,
    pub created_at: sqlx::types::time::OffsetDateTime,
}

impl From<Registration> for client::output::Registration {
    fn from(registration: Registration) -> Self {
        let Registration {
            id,
            client_id,
            machine_id,
            common_access_card_id,
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
            registration_request_id,
            election_id,
            precinct_id,
            ballot_style_id,
            created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Ballot {
    pub id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub common_access_card_id: String,
    pub registration_id: ServerId,
    pub cast_vote_record: Json<crate::cvr::Cvr>,
    pub created_at: sqlx::types::time::OffsetDateTime,
}

impl From<Ballot> for client::output::Ballot {
    fn from(ballot: Ballot) -> Self {
        let Ballot {
            id,
            client_id,
            machine_id,
            common_access_card_id,
            registration_id,
            cast_vote_record,
            created_at,
        } = ballot;

        Self {
            server_id: id,
            client_id,
            machine_id,
            common_access_card_id,
            registration_id,
            cast_vote_record: cast_vote_record.0,
            created_at,
        }
    }
}

pub(crate) async fn get_admins(
    db: &mut PoolConnection<Postgres>,
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
    since_election_id: Option<ServerId>,
) -> Result<Vec<Election>, sqlx::Error> {
    let since_election = match since_election_id {
        Some(id) => sqlx::query!(
            r#"
            SELECT created_at
            FROM elections
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut *db)
        .await
        .ok(),
        None => None,
    }
    .flatten();

    match since_election {
        Some(election) => {
            sqlx::query_as!(
                Election,
                r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    election as "election: _",
                    created_at
                FROM elections
                WHERE created_at > $1
                ORDER BY created_at DESC
                "#,
                election.created_at
            )
            .fetch_all(&mut *db)
            .await
        }
        None => {
            sqlx::query_as!(
                Election,
                r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    election as "election: _",
                    created_at
                FROM elections
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&mut *db)
            .await
        }
    }
}

pub(crate) async fn get_registration_requests(
    db: &mut PoolConnection<Postgres>,
    since_registration_request_id: Option<ServerId>,
) -> Result<Vec<RegistrationRequest>, sqlx::Error> {
    let since_registration_request = match since_registration_request_id {
        Some(id) => {
            sqlx::query!(
                r#"
                SELECT created_at
                FROM registration_requests
                WHERE id = $1
                "#,
                id.0
            )
            .fetch_optional(&mut *db)
            .await?
        }
        None => None,
    };

    match since_registration_request {
        Some(registration_request) => {
            sqlx::query_as!(
                RegistrationRequest,
                r#"
                SELECT
                    id as "id: ServerId",
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
                    state_id,
                    created_at
                FROM registration_requests
                WHERE created_at > $1
                ORDER BY created_at DESC
                "#,
                registration_request.created_at
            )
            .fetch_all(&mut *db)
            .await
        }
        None => {
            sqlx::query_as!(
                RegistrationRequest,
                r#"
                SELECT
                    id as "id: ServerId",
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
                    state_id,
                    created_at
                FROM registration_requests
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&mut *db)
            .await
        }
    }
}

pub(crate) async fn get_registrations(
    db: &mut PoolConnection<Postgres>,
    since_registration_id: Option<ServerId>,
) -> Result<Vec<Registration>, sqlx::Error> {
    let since_registration = match since_registration_id {
        Some(registration_id) => sqlx::query!(
            r#"
        SELECT created_at
        FROM registrations
        WHERE id = $1
        "#,
            registration_id.0
        )
        .fetch_optional(&mut *db)
        .await
        .ok()
        .flatten(),
        None => None,
    };

    match since_registration {
        Some(registration) => {
            sqlx::query_as!(
                Registration,
                r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
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
            .fetch_all(&mut *db)
            .await
        }
        None => {
            sqlx::query_as!(
                Registration,
                r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
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
            .fetch_all(&mut *db)
            .await
        }
    }
}

pub(crate) async fn add_registration_request_from_voter_terminal(
    db: &mut PoolConnection<Postgres>,
    request: &client::input::RegistrationRequest,
) -> Result<ServerId, sqlx::Error> {
    let registration_request_id = ServerId::new();

    sqlx::query!(
        r#"
        INSERT INTO registration_requests (
            id,
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
            state_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        registration_request_id.0,
        request.client_id.0,
        request.machine_id,
        request.common_access_card_id,
        request.given_name,
        request.family_name,
        request.address_line_1,
        request.address_line_2,
        request.city,
        request.state,
        request.postal_code,
        request.state_id
    )
    .execute(db)
    .await?;

    Ok(registration_request_id)
}

pub(crate) async fn add_election_from_voter_terminal(
    db: &mut PoolConnection<Postgres>,
    election: client::input::Election,
) -> Result<ServerId, sqlx::Error> {
    let election_id = ServerId::new();

    sqlx::query!(
        r#"
        INSERT INTO elections (
            id,
            client_id,
            machine_id,
            election
        )
        VALUES ($1, $2, $3, $4)
        "#,
        election_id.0,
        election.client_id.0,
        election.machine_id,
        Json(election.election) as _
    )
    .execute(db)
    .await?;

    Ok(election_id)
}

pub(crate) async fn add_registration_from_voter_terminal(
    db: &mut PoolConnection<Postgres>,
    registration: client::input::Registration,
) -> Result<ServerId, sqlx::Error> {
    let registration_request = sqlx::query!(
        r#"
        SELECT
            id as "id: ServerId"
        FROM registration_requests
        WHERE client_id = $1
        AND machine_id = $2
        AND common_access_card_id = $3
        "#,
        registration.registration_request_id.0,
        registration.machine_id,
        registration.common_access_card_id
    )
    .fetch_one(&mut *db)
    .await
    .map_err(|err| {
        eprintln!("unable to find registration: {:?}", registration);
        err
    })?;
    let election = sqlx::query!(
        r#"
        SELECT
            id as "id: ServerId"
        FROM elections
        WHERE client_id = $1
        AND machine_id = $2
        "#,
        registration.election_id.0,
        registration.machine_id,
    )
    .fetch_one(&mut *db)
    .await
    .map_err(|err| {
        eprintln!("unable to find election: {:?}", registration);
        err
    })?;

    let registration_id = ServerId::new();

    sqlx::query!(
        r#"
        INSERT INTO registrations (
            id,
            client_id,
            machine_id,
            common_access_card_id,
            registration_request_id,
            election_id,
            precinct_id,
            ballot_style_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        registration_id.0,
        registration.client_id.0,
        registration.machine_id,
        registration.common_access_card_id,
        registration_request.id.0,
        election.id.0,
        registration.precinct_id,
        registration.ballot_style_id
    )
    .execute(&mut *db)
    .await?;

    Ok(registration_id)
}

pub(crate) async fn add_ballot_from_voter_terminal(
    db: &mut PoolConnection<Postgres>,
    ballot: client::input::Ballot,
) -> Result<ServerId, sqlx::Error> {
    // we want to find the associated registration for this ballot
    let registration = sqlx::query!(
        r#"
        SELECT id
        FROM registrations
        WHERE client_id = $1
        AND machine_id = $2
        AND common_access_card_id = $3
        "#,
        ballot.registration_id.0,
        ballot.machine_id,
        ballot.common_access_card_id
    )
    .fetch_one(&mut *db)
    .await?;

    // we want to insert the ballot, but use the registration id we just found
    let ballot_id = ServerId::new();

    sqlx::query!(
        r#"
        INSERT INTO ballots (
            id,
            client_id,
            machine_id,
            common_access_card_id,
            registration_id,
            cast_vote_record
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        ballot_id.0,
        ballot.client_id.0,
        ballot.machine_id,
        ballot.common_access_card_id,
        registration.id,
        Json(ballot.cast_vote_record) as _
    )
    .execute(&mut *db)
    .await?;

    Ok(ballot_id)
}

pub(crate) async fn get_ballots(
    db: &mut PoolConnection<Postgres>,
    since_ballot_id: Option<ServerId>,
) -> Result<Vec<Ballot>, sqlx::Error> {
    let since_ballot = match since_ballot_id {
        Some(id) => sqlx::query!(
            r#"
            SELECT
                created_at
            FROM ballots
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut *db)
        .await
        .ok()
        .flatten(),
        None => None,
    };

    match since_ballot {
        Some(ballot) => {
            sqlx::query_as!(
                Ballot,
                r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    common_access_card_id,
                    registration_id as "registration_id: ServerId",
                    cast_vote_record as "cast_vote_record: _",
                    created_at
                FROM ballots
                WHERE created_at > $1
                ORDER BY created_at DESC
                "#,
                ballot.created_at
            )
            .fetch_all(&mut *db)
            .await
        }
        None => {
            sqlx::query_as!(
                Ballot,
                r#"
                SELECT
                    id as "id: ServerId",
                    client_id as "client_id: ClientId",
                    machine_id,
                    common_access_card_id,
                    registration_id as "registration_id: ServerId",
                    cast_vote_record as "cast_vote_record: _",
                    created_at
                FROM ballots
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&mut *db)
            .await
        }
    }
}
