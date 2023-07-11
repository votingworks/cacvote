#[macro_use]
extern crate rocket;

use db::run_migrations;
use rocket::fairing;
use rocket_db_pools::Database;
use routes::*;

mod cvr;
mod db;
mod routes;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(db::Db::init())
        .attach(fairing::AdHoc::try_on_ignite(
            "Run Migrations",
            run_migrations,
        ))
        .mount(
            "/",
            routes![
                create_election,
                get_elections,
                get_voters,
                parse_cast_vote_record_report,
                rave_mark_sync,
            ],
        )
}
