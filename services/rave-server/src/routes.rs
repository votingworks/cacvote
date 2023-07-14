use crate::client::ServerId;
use crate::cvr::Election;
use crate::{client, db};
use rocket::http::{ContentType, Status};
use rocket::serde::json::{json, Json};
use rocket_db_pools::{sqlx, Connection};
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetElectionsInput {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetElectionsOutput {
    elections: Vec<db::Election>,
}

#[post("/api/getElections", format = "json", data = "<_input>")]
pub(crate) async fn get_elections(
    mut db: Connection<db::Db>,
    _input: Json<GetElectionsInput>,
) -> (Status, (ContentType, String)) {
    let elections = match db::get_elections(&mut db, None).await {
        Err(e) => {
            return (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({ "error": e.to_string() }).to_string(),
                ),
            )
        }
        Ok(elections) => elections,
    };

    let output = GetElectionsOutput { elections };

    (
        Status::Ok,
        (ContentType::JSON, serde_json::to_string(&output).unwrap()),
    )
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateElectionInput {
    id: Uuid,
    election_data: String,
}

#[post("/api/createElection", format = "json", data = "<input>")]
pub(crate) async fn create_election(
    mut db: Connection<db::Db>,
    input: Json<CreateElectionInput>,
) -> (Status, (ContentType, String)) {
    let id = input.id;
    let election_data_parse_result: Result<Election, _> =
        serde_json::from_str(&input.election_data);

    let election = match election_data_parse_result {
        Ok(election) => election,
        Err(e) => {
            return (
                Status::BadRequest,
                (
                    ContentType::JSON,
                    json!({ "error": e.to_string() }).to_string(),
                ),
            )
        }
    };

    match db::create_election(&mut db, id, election).await {
        Err(e) => (
            Status::InternalServerError,
            (
                ContentType::JSON,
                json!({ "error": e.to_string() }).to_string(),
            ),
        ),
        Ok(_) => (Status::Created, (ContentType::JSON, json!({}).to_string())),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaveMarkSyncInput {
    #[serde(default)]
    last_synced_registration_request_id: Option<ServerId>,
    #[serde(default)]
    last_synced_registration_id: Option<ServerId>,
    #[serde(default)]
    last_synced_election_id: Option<ServerId>,
    #[serde(default)]
    last_synced_ballot_id: Option<ServerId>,
    #[serde(default)]
    registration_requests: Vec<client::input::RegistrationRequest>,
    #[serde(default)]
    elections: Vec<client::input::Election>,
    #[serde(default)]
    registrations: Vec<client::input::Registration>,
    #[serde(default)]
    ballots: Vec<client::input::Ballot>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RaveMarkSyncOutput {
    admins: Vec<client::output::Admin>,
    elections: Vec<client::output::Election>,
    registration_requests: Vec<client::output::RegistrationRequest>,
    registrations: Vec<client::output::Registration>,
    ballots: Vec<client::output::Ballot>,
}

#[post("/api/sync", format = "json", data = "<input>")]
pub(crate) async fn rave_mark_sync(
    mut db: Connection<db::Db>,
    input: Json<RaveMarkSyncInput>,
) -> (Status, (ContentType, String)) {
    let RaveMarkSyncInput {
        last_synced_registration_request_id,
        last_synced_registration_id,
        last_synced_election_id,
        last_synced_ballot_id,
        registration_requests,
        elections,
        registrations,
        ballots,
    } = input.into_inner();

    for client_request in registration_requests.into_iter() {
        let server_request: client::input::RegistrationRequest = client_request;
        let result =
            db::add_registration_request_from_voter_terminal(&mut db, &server_request).await;

        if let Err(e) = result {
            error!("Failed to insert registration request: {}", e);
        }
    }

    for election in elections.into_iter() {
        let result = db::add_election_from_voter_terminal(&mut db, election).await;

        if let Err(e) = result {
            error!("Failed to insert election: {}", e);
        }
    }

    for registration in registrations.into_iter() {
        let result = db::add_registration_from_voter_terminal(&mut db, registration).await;

        if let Err(e) = result {
            error!("Failed to insert registration: {}", e);
        }
    }

    for ballot in ballots.into_iter() {
        let result = db::add_ballot_from_voter_terminal(&mut db, ballot).await;

        if let Err(e) = result {
            error!("Failed to insert ballot: {}", e);
        }
    }

    let get_admins_result = db::get_admins(&mut db).await;
    let admins = match get_admins_result {
        Err(e) => {
            return (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({ "error": e.to_string() }).to_string(),
                ),
            )
        }
        Ok(admins) => admins,
    };

    let get_elections_result = db::get_elections(&mut db, last_synced_election_id).await;
    let elections = match get_elections_result {
        Err(e) => {
            return (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({ "error": e.to_string() }).to_string(),
                ),
            )
        }
        Ok(elections) => elections,
    };

    let get_registration_requests_result =
        db::get_registration_requests(&mut db, last_synced_registration_request_id).await;
    let registration_requests = match get_registration_requests_result {
        Err(e) => {
            return (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({ "error": e.to_string() }).to_string(),
                ),
            )
        }
        Ok(registration_requests) => registration_requests,
    };

    let get_registrations_result =
        db::get_registrations(&mut db, last_synced_registration_id).await;
    let registrations = match get_registrations_result {
        Err(e) => {
            return (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({ "error": e.to_string() }).to_string(),
                ),
            )
        }
        Ok(registrations) => registrations,
    };

    let ballots = match db::get_ballots(&mut db, last_synced_ballot_id).await {
        Err(e) => {
            return (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({ "error": e.to_string() }).to_string(),
                ),
            )
        }
        Ok(ballots) => ballots,
    };

    let output = RaveMarkSyncOutput {
        admins: admins.into_iter().map(|admin| admin.into()).collect(),
        elections: elections
            .into_iter()
            .map(|election| election.into())
            .collect(),
        registration_requests: registration_requests
            .into_iter()
            .map(|registration_request| registration_request.into())
            .collect(),
        registrations: registrations
            .into_iter()
            .map(|registration| registration.into())
            .collect(),
        ballots: ballots.into_iter().map(|ballot| ballot.into()).collect(),
    };

    (
        Status::Ok,
        (ContentType::JSON, serde_json::to_string(&output).unwrap()),
    )
}
