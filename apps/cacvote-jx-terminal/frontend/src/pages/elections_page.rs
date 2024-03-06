#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use ui_rs::FileButton;

use crate::util::file::read_file_as_bytes;

pub fn ElectionsPage(cx: Scope) -> Element {
    render! (
        div {
                h1 { class: "text-2xl font-bold mb-4", "Elections" }
                rsx!(div { "No elections found." })
                FileButton {
                        "Import Election",
                        class: "mt-4",
                        onfile: move |file_engine: Arc<dyn FileEngine>| {
                            cx.spawn({
                                to_owned![file_engine];
                                async move {
                                    if let Some(election_data) = read_file_as_bytes(file_engine).await {
                                        log::info!("uploading election: {election_data:?}");
                                    };
                                }
                            });
                        },
                }
            }
    )
}
