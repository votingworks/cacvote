#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::route::Route;

pub fn App(cx: Scope) -> Element {
    render!(Router::<Route> {})
}
