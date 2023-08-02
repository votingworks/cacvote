use rocket::fs::{relative, NamedFile};
use rocket::http::{ContentType, Status};
use rocket::serde::json::json;
use rocket_db_pools::Connection;

use crate::db;
use crate::sync::sync;

#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(relative!("static/index.html")).await.ok()
}

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
