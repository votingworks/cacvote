#![allow(non_snake_case)]

use log::LevelFilter;

mod app;
mod components;
mod layout;
mod pages;
mod route;
mod util;

fn main() {
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    console_error_panic_hook::set_once();

    log::info!("starting app");
    dioxus_web::launch(crate::app::App);
}
