#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use types_rs::rave::jx;
use ui_rs::{DateOrDateTimeCell, FileButton};

use crate::util::{file::read_file_as_bytes, url::get_url};

#[derive(PartialEq, Props)]
pub struct ElectionsPageProps {
    jurisdiction_id: String,
}

pub fn ElectionsPage(cx: Scope<ElectionsPageProps>) -> Element {
    let app_data = use_shared_state::<jx::LoggedInAppData>(cx).unwrap();
    let elections = &app_data.read().elections;
    let is_uploading = use_state(cx, || false);
    let jurisdiction_id = &cx.props.jurisdiction_id;
    let upload_election = {
        to_owned![is_uploading, jurisdiction_id];
        |election_data: Vec<u8>| async move {
            is_uploading.set(true);

            let url = get_url(&*format!(
                "/api/elections?jurisdiction_id={jurisdiction_id}"
            ));
            let client = reqwest::Client::new();
            let res = client
                .post(url)
                .body(election_data)
                .header("Content-Type", "application/json")
                .send()
                .await;

            is_uploading.set(false);

            res
        }
    };

    render! (
        div {
                h1 { class: "text-2xl font-bold mb-4", "Elections" }
                if elections.is_empty() {
                    rsx!(div { "No elections found." })
                } else {
                    rsx!(table { class: "table-auto w-full",
                        thead {
                            tr {
                                th { class: "px-4 py-2 text-left", "Election ID" }
                                th { class: "px-4 py-2 text-left", "Title" }
                                th { class: "px-4 py-2 text-left", "Date" }
                                th { class: "px-4 py-2 text-left", "Synced" }
                                th { class: "px-4 py-2 text-left", "Created At" }
                            }
                        }
                        tbody {
                            for election in elections.iter() {
                                tr {
                                    td {
                                        class: "border px-4 py-2",
                                        title: "Database ID: {election.id}\n\nFull Election Hash: {election.election_hash}",
                                        "{election.election_hash.to_partial()}"
                                    }
                                    td { class: "border px-4 py-2", "{election.title}" }
                                    DateOrDateTimeCell {
                                        date_or_datetime: election.date,
                                    }
                                    td { class: "border px-4 py-2", if election.is_synced() { "Yes" } else { "No" } }
                                    DateOrDateTimeCell {
                                        date_or_datetime: election.created_at,
                                    }
                                }
                            }
                        }
                    })
                }
                FileButton {
                        "Import Election",
                        class: "mt-4",
                        onfile: move |file_engine: Arc<dyn FileEngine>| {
                            cx.spawn({
                                to_owned![upload_election, file_engine];
                                async move {
                                    if let Some(election_data) = read_file_as_bytes(file_engine).await {
                                        match upload_election(election_data).await {
                                            Ok(response) => {
                                                if !response.status().is_success() {
                                                    web_sys::window()
                                                        .unwrap()
                                                        .alert_with_message(format!("Error uploading election: {}", response.status().as_str()).as_str())
                                                        .unwrap();
                                                    return;
                                                }

                                                log::info!("uploaded election: {:?}", response);
                                            }
                                            Err(err) => {
                                                web_sys::window()
                                                    .unwrap()
                                                    .alert_with_message(format!("Error uploading election: {:?}", err).as_str())
                                                    .unwrap();
                                            }
                                        }
                                    };
                                }
                            });
                        },
                }
            }
    )
}
