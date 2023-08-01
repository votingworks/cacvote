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
mod routes;
mod sync;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(db::Db::init())
        .attach(fairing::AdHoc::try_on_ignite(
            "Run Migrations",
            run_migrations,
        ))
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![index, do_scan, do_sync, get_status])
}
