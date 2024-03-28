#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use types_rs::cacvote::SessionData;
use ui_rs::FileButton;

use crate::{
    route::Route,
    util::{file::read_file_as_bytes, url::get_url},
};

pub fn ElectionsPage(cx: Scope) -> Element {
    let nav = use_navigator(cx);
    let session_data = use_shared_state::<SessionData>(cx).unwrap();
    let session_data = &*session_data.read();
    let elections = match session_data {
        SessionData::Authenticated { elections, .. } => Some(elections),
        _ => None,
    };

    let is_uploading = use_state(cx, || false);
    let upload_election = {
        to_owned![is_uploading];
        |election_data: Vec<u8>| async move {
            is_uploading.set(true);

            log::info!(
                "election data: {}",
                String::from_utf8(election_data.clone()).unwrap()
            );

            let url = get_url("/api/elections");
            let client = reqwest::Client::new();
            let res = client.post(url).body(election_data).send().await;

            is_uploading.set(false);

            Some(res)
        }
    };

    use_effect(cx, (session_data,), |(session_data,)| {
        to_owned![nav, session_data];
        async move {
            if matches!(session_data, SessionData::Unauthenticated { .. }) {
                nav.push(Route::MachineLockedPage);
            }
        }
    });

    render! (
        div {
            h1 { class: "text-2xl font-bold mb-4", "Elections" }
            match elections.map(Vec::as_slice) {
                None | Some([]) => {
                    rsx!(div { "No elections found." })
                }
                Some(election_definitions) => {
                    rsx!(
                        table {
                            class: "table-auto w-full",
                            thead {
                                tr {
                                    th { class: "px-4 py-2 text-left", "Title" }
                                }
                            }
                            for election_definition in election_definitions.iter() {
                                tr {
                                    td { class: "border px-4 py-2 text-sm", "{election_definition.election.title}" }
                                }
                            }
                        }
                    )
                }
            },
            FileButton {
                class: "mt-4",
                onfile: move |file_engine: Arc<dyn FileEngine>| {
                    cx.spawn({
                        to_owned![upload_election, file_engine];
                        async move {
                            if let Some(election_data) = read_file_as_bytes(file_engine).await {
                                match upload_election(election_data).await {
                                    Some(Ok(response)) => {
                                        if !response.status().is_success() {
                                            web_sys::window()
                                                .unwrap()
                                                .alert_with_message(
                                                    &format!(
                                                        "Failed to upload election: {:?}",
                                                        response.text().await.unwrap_or("unknown error".to_owned()),
                                                    ),
                                                )
                                                .unwrap();
                                            return;
                                        }
                                        log::info!("Election uploaded successfully");
                                    }
                                    Some(Err(err)) => {
                                        log::error!("Failed to upload election: {err}");
                                        web_sys::window()
                                            .unwrap()
                                            .alert_with_message(
                                                &format!("Failed to upload election: {err}"),
                                            )
                                            .unwrap();
                                    }
                                    None => {
                                        log::error!("Invalid election data");
                                    }
                                }
                            }
                        }
                    });
                },
                "Import Election"
            }
        }
    )
}
