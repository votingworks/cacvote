use crate::cvr::{CastVoteRecordReport, Election};
use crate::db;
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVotersInput {
    since: Option<time::PrimitiveDateTime>,
}

#[post("/api/getVoters", format = "json", data = "<input>")]
pub(crate) async fn get_voters(
    mut db: Connection<db::Db>,
    input: Json<GetVotersInput>,
) -> (Status, (ContentType, String)) {
    let result = db::get_voters(&mut db, input.since).await;

    match result {
        Err(e) => (
            Status::InternalServerError,
            (
                ContentType::JSON,
                json!({ "error": e.to_string() }).to_string(),
            ),
        ),
        Ok(voters) => (Status::Ok, (ContentType::JSON, json!(voters).to_string())),
    }
}

#[post("/api/parseCastVoteRecordReport", format = "json", data = "<input>")]
pub(crate) async fn parse_cast_vote_record_report(
    input: Json<CastVoteRecordReport>,
) -> (Status, (ContentType, String)) {
    (
        Status::Ok,
        (
            ContentType::JSON,
            serde_json::to_string_pretty(&json!(*input)).unwrap(),
        ),
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaveMarkSyncInput {
    #[serde(default)]
    last_synced_election_id: Option<Uuid>,
    #[serde(default)]
    last_synced_voter_election_registration_id: Option<Uuid>,
    #[serde(default)]
    last_synced_voter_election_selection_id: Option<Uuid>,
    #[serde(default)]
    voter_registration_requests: Vec<db::VoterRegistrationRequest>,
    #[serde(default)]
    voter_election_selections: Vec<db::VoterElectionSelection>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RaveMarkSyncOutput {
    elections: Vec<db::Election>,
    voter_election_registrations: Vec<db::VoterElectionRegistration>,
    voter_election_selections: Vec<db::VoterElectionSelection>,
}

#[post("/api/rave-mark/sync", format = "json", data = "<input>")]
pub(crate) async fn rave_mark_sync(
    mut db: Connection<db::Db>,
    input: Json<RaveMarkSyncInput>,
) -> (Status, (ContentType, String)) {
    let RaveMarkSyncInput {
        last_synced_election_id,
        last_synced_voter_election_registration_id,
        last_synced_voter_election_selection_id,
        voter_registration_requests,
        voter_election_selections,
    } = input.into_inner();

    for request in voter_registration_requests.iter() {
        let result = db::add_voter_registration_request(&mut db, request).await;

        if let Err(e) = result {
            error!("Failed to insert voter registration request: {}", e);
        }
    }

    for voter_election_selection in voter_election_selections.iter() {
        let result = db::add_voter_election_selection(&mut db, voter_election_selection).await;

        if let Err(e) = result {
            error!("Failed to insert voter election selection: {}", e);
        }
    }

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

    let get_voter_election_registrations_result =
        db::get_voter_election_registrations(&mut db, last_synced_voter_election_registration_id)
            .await;
    let voter_election_registrations = match get_voter_election_registrations_result {
        Err(e) => {
            return (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({ "error": e.to_string() }).to_string(),
                ),
            )
        }
        Ok(voter_election_registrations) => voter_election_registrations,
    };

    let voter_election_selections =
        match db::get_voter_election_selections(&mut db, last_synced_voter_election_selection_id)
            .await
        {
            Err(e) => {
                return (
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        json!({ "error": e.to_string() }).to_string(),
                    ),
                )
            }
            Ok(voter_election_selections) => voter_election_selections,
        };

    let output = RaveMarkSyncOutput {
        elections,
        voter_election_registrations,
        voter_election_selections,
    };

    (
        Status::Ok,
        (ContentType::JSON, serde_json::to_string(&output).unwrap()),
    )
}
