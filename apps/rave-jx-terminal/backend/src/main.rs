#[macro_use]
extern crate rocket;

use db::run_migrations;
use rocket::fairing;
use rocket_db_pools::Database;
use routes::*;

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
        .mount("/", routes![index, get_status, do_sync])
}
