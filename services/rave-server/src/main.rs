#[macro_use]
extern crate rocket;

use db::run_migrations;
use rocket::{
    fairing,
    fs::{relative, FileServer},
};
use rocket_db_pools::Database;
use routes::*;

mod db;
mod routes;

#[launch]
fn rocket() -> _ {
    color_eyre::install().unwrap();

    rocket::build()
        .attach(db::Db::init())
        .attach(fairing::AdHoc::try_on_ignite(
            "Run Migrations",
            run_migrations,
        ))
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![get_status, do_sync, create_admin])
}
