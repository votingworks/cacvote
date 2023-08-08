#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::route::Route;

pub fn Layout(cx: Scope) -> Element {
    let elections_link_active_class = "bg-gray-300 dark:bg-gray-800";

    render!(
        div {
            class: "h-screen w-screen flex dark:bg-gray-800 dark:text-gray-300",
            div {
                class: "w-1/5 bg-gray-200 dark:bg-gray-700",
                ul {
                    class: "mt-8",
                    for route in [Route::ElectionsPage, Route::VotersPage] {
                        li {
                            Link {
                                to: route.clone(),
                                active_class: elections_link_active_class,
                                class: "px-4 py-2 block hover:bg-gray-300 dark:bg-gray-700 hover:dark:text-gray-700 hover:cursor-pointer",
                                "{route.label()}"
                            }
                        }
                    }
                    li {
                        class: "fixed bottom-0 w-1/5 font-bold text-center py-2",
                        "RAVE Jurisdiction"
                    }
                }
            }
            div { class: "w-4/5 p-8",
                Outlet::<Route> {}
            }
        }
    )
}
