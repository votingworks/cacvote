use std::time::Duration;

use rocket::http::{ContentType, Status};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::json;
use rocket::tokio::time::sleep;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use types_rs::election::ElectionDefinition;
use types_rs::rave::ClientId;

use crate::db::{self, Election};
use crate::sync::sync;

#[get("/api/status")]
pub(crate) async fn get_status() -> Status {
    Status::Ok
}

#[post("/api/sync")]
pub(crate) async fn do_sync(mut db: Connection<db::Db>) -> (Status, (ContentType, String)) {
    match sync(&mut db).await {
        Ok(_) => (
            Status::Ok,
            (ContentType::JSON, json!({ "success": true }).to_string()),
        ),
        Err(e) => (
            Status::InternalServerError,
            (
                ContentType::JSON,
                json!({
                    "success": false,
                    "error": format!("failed to sync with RAVE server: {}", e)
                })
                .to_string(),
            ),
        ),
    }
}

#[derive(Debug, Serialize)]
struct StatusStreamEvent {
    elections: Vec<Election>,
}

#[get("/api/status-stream")]
pub(crate) async fn get_status_stream(mut db: Connection<db::Db>) -> EventStream![Event] {
    let mut last_elections = vec![];
    let mut last_registration_requests = vec![];
    let mut last_registrations = vec![];

    EventStream! {
        loop {
            let elections = match db::get_elections(&mut db, None).await {
                Ok(elections) => elections,
                Err(e) => {
                    error!("failed to get elections: {}", e);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            let registration_requests = match db::get_registration_requests(&mut db).await {
                Ok(registration_requests) => registration_requests,
                Err(e) => {
                    error!("failed to get registration requests: {}", e);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            let registrations = match db::get_registrations(&mut db).await {
                Ok(registrations) => registrations,
                Err(e) => {
                    error!("failed to get registrations: {}", e);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            if elections != last_elections || registration_requests != last_registration_requests || registrations != last_registrations {
                yield Event::json(&json!({
                    "elections": elections,
                    "registrationRequests": registration_requests,
                    "registrations": registrations,
                }));
                last_elections = elections;
                last_registration_requests = registration_requests;
                last_registrations = registrations;
            }

            sleep(Duration::from_secs(1)).await;
        }
    }
}

#[post("/api/elections", format = "json", data = "<election>")]
pub(crate) async fn create_election(
    mut db: Connection<db::Db>,
    election: String,
) -> (Status, (ContentType, String)) {
    let election_definition: ElectionDefinition = match election.parse() {
        Ok(e) => e,
        Err(e) => {
            error!("failed to parse election: {}", e);
            return (
                Status::BadRequest,
                (
                    ContentType::JSON,
                    json!({
                        "success": false,
                        "error": format!("failed to parse election: {}", e)
                    })
                    .to_string(),
                ),
            );
        }
    };

    match db::add_election(&mut db, election_definition).await {
        Ok(_) => (
            Status::Ok,
            (ContentType::JSON, json!({ "success": true }).to_string()),
        ),
        Err(e) => {
            error!("failed to create election: {}", e);
            (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({
                        "success": false,
                        "error": format!("failed to create election: {}", e)
                    })
                    .to_string(),
                ),
            )
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinkVoterRegistrationRequestAndElectionRequest {
    election_id: ClientId,
    registration_request_id: ClientId,
}

#[post("/api/registrations", format = "json", data = "<registration>")]
pub(crate) async fn create_registration(
    mut db: Connection<db::Db>,
    registration: String,
) -> (Status, (ContentType, String)) {
    let registration: LinkVoterRegistrationRequestAndElectionRequest =
        match serde_json::from_str(&registration) {
            Ok(r) => r,
            Err(e) => {
                error!("failed to parse registration: {}", e);
                return (
                    Status::BadRequest,
                    (
                        ContentType::JSON,
                        json!({
                            "success": false,
                            "error": format!("failed to parse registration: {}", e)
                        })
                        .to_string(),
                    ),
                );
            }
        };

    let election = match db::get_elections(&mut db, None).await {
        Ok(elections) => elections
            .into_iter()
            .find(|e| e.id == registration.election_id),
        Err(e) => {
            error!("failed to get elections: {}", e);
            return (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({
                        "success": false,
                        "error": format!("failed to get elections: {}", e)
                    })
                    .to_string(),
                ),
            );
        }
    };

    let Some(election) = election else {
        error!("election not found");
        return (
            Status::BadRequest,
            (
                ContentType::JSON,
                json!({
                    "success": false,
                    "error": "election not found"
                })
                .to_string(),
            ),
        );
    };

    // TODO: support choosing a ballot style and precinct
    let ballot_style = &election.definition.election.ballot_styles[0];
    let precinct_id = &ballot_style.precincts[0];

    match db::create_registration(
        &mut db,
        registration.registration_request_id,
        registration.election_id,
        precinct_id,
        &ballot_style.id,
    )
    .await
    {
        Ok(_) => (
            Status::Ok,
            (ContentType::JSON, json!({ "success": true }).to_string()),
        ),
        Err(e) => {
            error!("failed to create registration: {}", e);
            (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({
                        "success": false,
                        "error": format!("failed to create registration: {}", e)
                    })
                    .to_string(),
                ),
            )
        }
    }
}
