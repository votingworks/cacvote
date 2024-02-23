#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use types_rs::cacvote::jx;
use ui_rs::{DateOrDateTimeCell, FileButton};

use crate::{
    route::Route,
    util::{file::read_file_as_bytes, url::get_url},
};

pub fn ElectionsPage(cx: Scope) -> Element {
    let nav = use_navigator(cx);
    let app_data = use_shared_state::<jx::AppData>(cx).unwrap();
    let app_data = &*app_data.read();
    let is_logged_in = matches!(app_data, jx::AppData::LoggedIn { .. });
    let elections = if let jx::AppData::LoggedIn { app_data, .. } = app_data {
        Some(&app_data.elections)
    } else {
        None
    };
    let is_uploading = use_state(cx, || false);
    let upload_election = {
        to_owned![is_uploading];
        |election_data: Vec<u8>| async move {
            is_uploading.set(true);

            let url = get_url("/api/elections");
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

    use_effect(cx, (&is_logged_in,), |(is_logged_in,)| {
        to_owned![nav, is_logged_in];
        async move {
            if !is_logged_in {
                nav.push(Route::MachineLockedPage);
            }
        }
    });

    render! (
        div {
                h1 { class: "text-2xl font-bold mb-4", "Elections" }
                match elections {
                    Some(elections) => {
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
                    }
                    None => {
                        rsx!(div { "Loading elections…" })
                    }
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
                                                        .alert_with_message(format!("Error uploading election: {}", response.text().await.unwrap()).as_str())
                                                        .unwrap();
                                                    return;
                                                }

                                                log::info!("uploaded election: {:?}", response);
                                            }
                                            Err(err) => {
                                                log::error!("error uploading election: {err}");

                                                web_sys::window()
                                                    .unwrap()
                                                    .alert_with_message(format!("Error uploading election: {err:?}").as_str())
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
