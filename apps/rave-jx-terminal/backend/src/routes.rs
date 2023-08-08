use std::time::Duration;

use rocket::http::{ContentType, Status};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::json;
use rocket::tokio::time::sleep;
use rocket_db_pools::Connection;
use types_rs::election::{BallotStyleId, ElectionDefinition, PrecinctId};
use types_rs::rave::jx;

use crate::db;
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

async fn get_app_data(executor: &mut sqlx::PgConnection) -> color_eyre::Result<jx::AppData> {
    let elections = db::get_elections(executor, None).await?;
    let registration_requests = db::get_registration_requests(executor).await?;
    let registrations = db::get_registrations(executor).await?;

    Ok(jx::AppData {
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
    })
}

#[get("/api/status-stream")]
pub(crate) async fn get_status_stream(mut db: Connection<db::Db>) -> EventStream![Event] {
    let mut last_app_data = jx::AppData::default();

    EventStream! {
        loop {
            let new_app_data = match get_app_data(&mut db).await {
                Ok(app_data) => app_data,
                Err(e) => {
                    error!("failed to get app data: {}", e);
                    continue;
                }
            };

            if new_app_data != last_app_data {
                yield Event::json(&new_app_data);
                last_app_data = new_app_data;
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

#[post("/api/registrations", format = "json", data = "<registration>")]
pub(crate) async fn create_registration(
    mut db: Connection<db::Db>,
    registration: String,
) -> (Status, (ContentType, String)) {
    let registration: jx::CreateRegistrationData = match serde_json::from_str(&registration) {
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

    let ballot_style_id = registration.ballot_style_id;
    let precinct_id = registration.precinct_id;

    match db::create_registration(
        &mut db,
        registration.registration_request_id,
        registration.election_id,
        &precinct_id,
        &ballot_style_id,
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
