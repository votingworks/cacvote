#[macro_use]
extern crate rocket;

use db::run_migrations;
use env::{RAVE_URL, VX_MACHINE_ID};
use rocket::{
    fairing,
    fs::{relative, FileServer},
};
use rocket_db_pools::Database;
use routes::*;

mod cac;
mod db;
mod env;
mod routes;
mod sync;

#[launch]
fn rocket() -> _ {
    color_eyre::install().unwrap();

    assert!(!VX_MACHINE_ID.is_empty(), "VX_MACHINE_ID must be set");
    assert!(!RAVE_URL.to_string().is_empty(), "RAVE_URL must be set");

    let dist_path = relative!("../frontend/dist");
    let _ = std::fs::create_dir_all(&dist_path);

    rocket::build()
        .attach(db::Db::init())
        .attach(fairing::AdHoc::try_on_ignite(
            "Run Migrations",
            run_migrations,
        ))
        .attach(fairing::AdHoc::try_on_ignite(
            "Sync with RAVE server periodically",
            sync::sync_periodically,
        ))
        .mount(
            "/",
            routes![
                get_status,
                get_status_stream,
                do_sync,
                create_election,
                create_registration
            ],
        )
        .mount("/", FileServer::from(relative!("../frontend/dist")))
}
