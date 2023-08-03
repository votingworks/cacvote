use rocket::http::{ContentType, Status};
use rocket::serde::json::json;
use rocket_db_pools::Connection;
use types_rs::election::ElectionDefinition;

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
