use crate::db;
use rocket::http::{ContentType, Status};
use rocket::serde::json::{json, Json};
use rocket_db_pools::Connection;
use sqlx::Acquire;
use types_rs::rave::{client, RaveServerSyncInput, RaveServerSyncOutput};

#[get("/api/status")]
pub(crate) async fn get_status() -> Status {
    Status::Ok
}

#[post("/api/sync", format = "json", data = "<input>")]
pub(crate) async fn do_sync(
    mut db: Connection<db::Db>,
    input: Json<RaveServerSyncInput>,
) -> (Status, (ContentType, String)) {
    let mut txn = match db.begin().await {
        Ok(txn) => txn,
        Err(e) => {
            error!("Failed to begin transaction: {}", e);
            return (
                Status::InternalServerError,
                (
                    ContentType::JSON,
                    json!({
                        "success": false,
                        "error": format!("failed to begin transaction: {}", e)
                    })
                    .to_string(),
                ),
            );
        }
    };

    let RaveServerSyncInput {
        last_synced_registration_request_id,
        last_synced_registration_id,
        last_synced_election_id,
        last_synced_printed_ballot_id,
        last_synced_scanned_ballot_id,
        registration_requests,
        elections,
        registrations,
        printed_ballots,
        scanned_ballots,
    } = input.into_inner();

    for client_request in registration_requests.into_iter() {
        let server_request: client::input::RegistrationRequest = client_request;
        let result = db::add_registration_request_from_client(&mut txn, &server_request).await;

        if let Err(e) = result {
            error!("Failed to insert registration request: {}", e);
        }
    }

    for election in elections.into_iter() {
        let result = db::add_election(&mut txn, election).await;

        if let Err(e) = result {
            error!("Failed to insert election: {}", e);
        }
    }

    for registration in registrations.into_iter() {
        let result = db::add_registration_from_client(&mut txn, registration).await;

        if let Err(e) = result {
            error!("Failed to insert registration: {}", e);
        }
    }

    for printed_ballot in printed_ballots.into_iter() {
        let result = db::add_printed_ballot_from_client(&mut txn, printed_ballot).await;

        if let Err(e) = result {
            error!("Failed to insert printed ballot: {}", e);
        }
    }

    for scanned_ballot in scanned_ballots.into_iter() {
        let result = db::add_scanned_ballot_from_client(&mut txn, scanned_ballot).await;

        if let Err(e) = result {
            error!("Failed to insert scanned ballot: {}", e);
        }
    }

    let get_admins_result = db::get_admins(&mut txn).await;
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

    let get_elections_result = db::get_elections(&mut txn, last_synced_election_id).await;
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
        db::get_registration_requests(&mut txn, last_synced_registration_request_id).await;
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
        db::get_registrations(&mut txn, last_synced_registration_id).await;
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

    let printed_ballots =
        match db::get_printed_ballots(&mut txn, last_synced_printed_ballot_id).await {
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

    let scanned_ballots =
        match db::get_scanned_ballots(&mut txn, last_synced_scanned_ballot_id).await {
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

    let output = RaveServerSyncOutput {
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
        printed_ballots: printed_ballots
            .into_iter()
            .map(|ballot| ballot.into())
            .collect(),
        scanned_ballots: scanned_ballots
            .into_iter()
            .map(|ballot| ballot.into())
            .collect(),
    };

    if let Err(err) = txn.commit().await {
        error!("Failed to commit transaction: {}", err);
        return (
            Status::InternalServerError,
            (
                ContentType::JSON,
                json!({ "error": err.to_string() }).to_string(),
            ),
        );
    }

    (
        Status::Ok,
        (ContentType::JSON, serde_json::to_string(&output).unwrap()),
    )
}

#[post("/api/admins", format = "json", data = "<input>")]
pub(crate) async fn create_admin(
    mut db: Connection<db::Db>,
    input: Json<client::input::Admin>,
) -> (Status, (ContentType, String)) {
    let input = input.into_inner();
    let result = db::add_admin(&mut db, input).await;

    match result {
        Ok(_) => (Status::Created, (ContentType::JSON, json!({}).to_string())),
        Err(e) => (
            Status::InternalServerError,
            (
                ContentType::JSON,
                json!({ "error": e.to_string() }).to_string(),
            ),
        ),
    }
}
