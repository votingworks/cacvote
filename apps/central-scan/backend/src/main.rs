#[macro_use]
extern crate rocket;

mod cards;
mod routes;

use routes::*;

#[launch]
fn rocket() -> _ {
    rocket::build()
        // .attach(db::Db::init())
        // .attach(fairing::AdHoc::try_on_ignite(
        //     "Run Migrations",
        //     run_migrations,
        // ))
        .mount("/", routes![hello_world, do_scan])
}
