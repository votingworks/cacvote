#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use types_rs::{cacvote, election};
use ui_rs::FileButton;

use crate::{
    route::Route,
    util::{file::read_file_as_bytes, url::get_url},
};

pub fn ElectionsPage(cx: Scope) -> Element {
    let nav = use_navigator(cx);
    let session_data = use_shared_state::<cacvote::SessionData>(cx).unwrap();
    let session_data = &*session_data.read();
    let (elections, jurisdiction_code) = match session_data {
        cacvote::SessionData::Authenticated {
            elections,
            jurisdiction_code,
            ..
        } => (Some(elections), Some(jurisdiction_code)),
        _ => (None, None),
    };

    let is_uploading = use_state(cx, || false);
    let mailing_address = use_state(cx, || "".to_owned());

    let upload_election = {
        to_owned![is_uploading, mailing_address];
        |election_data: Vec<u8>, jurisdiction_code: cacvote::JurisdictionCode| async move {
            is_uploading.set(true);

            log::info!("election data: {}", String::from_utf8_lossy(&election_data));

            let Ok(election_definition) =
                election::ElectionDefinition::try_from(election_data.as_slice())
            else {
                return None;
            };

            let url = get_url("/api/elections");
            let client = reqwest::Client::new();
            let election = cacvote::Election {
                election_definition,
                jurisdiction_code,
                mailing_address: mailing_address.get().clone(),
                electionguard_election_metadata_blob: vec![],
            };
            let res = client.post(url).json(&election).send().await;

            is_uploading.set(false);
            mailing_address.set("".to_owned());

            Some(res)
        }
    };

    use_effect(cx, (session_data,), |(session_data,)| {
        to_owned![nav, session_data];
        async move {
            if matches!(session_data, cacvote::SessionData::Unauthenticated { .. }) {
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
            h2 { class: "text-xl font-bold mt-8", "New Election" }
            textarea {
                class: "mt-4 w-30 p-2 border block",
                rows: 3,
                value: mailing_address.get().as_str(),
                oninput: move |e| {
                    mailing_address.set(e.inner().value.clone());
                },
                placeholder: "Mailing Address",
            },
            if let Some(jurisdiction_code) = jurisdiction_code.cloned() {
                rsx!(FileButton {
                    class: "mt-4",
                    disabled: mailing_address.get().chars().all(char::is_whitespace),
                    onfile: move |file_engine: Arc<dyn FileEngine>| {
                        cx.spawn({
                            to_owned![upload_election, file_engine, jurisdiction_code];
                            async move {
                                if let Some(election_data) = read_file_as_bytes(file_engine).await {
                                    match upload_election(election_data, jurisdiction_code).await {
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
                })
            }
        }
    )
}
