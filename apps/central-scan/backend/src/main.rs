#[macro_use]
extern crate rocket;

use db::run_migrations;
use rocket::{
    fairing,
    fs::{relative, FileServer},
};
use rocket_db_pools::Database;
use routes::*;

mod cards;
mod db;
mod env;
mod routes;
mod sync;

#[launch]
fn rocket() -> _ {
    color_eyre::install().unwrap();

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
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![index, do_scan, do_sync, get_status])
}
