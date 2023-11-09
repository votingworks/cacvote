use dioxus::prelude::*;
use types_rs::rave::jx;
use ui_rs::DateOrDateTimeCell;

use crate::{components::ElectionConfigurationCell, util::url::get_url};

pub fn VotersPage(cx: Scope) -> Element {
    let app_data = use_shared_state::<jx::LoggedInAppData>(cx).unwrap();
    let app_data = app_data.read();
    let elections = app_data.elections.clone();
    let registration_requests = app_data.registration_requests.clone();
    let registrations = app_data.registrations.clone();
    let pending_registration_requests = registration_requests
        .iter()
        .filter(|registration_request| {
            !registrations
                .iter()
                .any(|registration| registration.is_registration_request(registration_request))
        })
        .map(Clone::clone)
        .collect::<Vec<_>>();

    render!(
        h1 { class: "text-2xl font-bold mb-4", "Pending Registrations" }
        if pending_registration_requests.is_empty() {
            rsx!("No pending registrations")
        } else {
            rsx!(PendingRegistrationsTable {
                elections: elections,
                pending_registration_requests: pending_registration_requests,
            })
        }

        h1 { class: "text-2xl font-bold mt-4 mb-4", "Registrations" }
        if registrations.is_empty() {
            rsx!("No registrations")
        } else {
            rsx!(RegistrationsTable {
                registrations: registrations,
            })
        }
    )
}

#[derive(PartialEq, Props)]
struct PendingRegistrationsTableProps {
    elections: Vec<jx::Election>,
    pending_registration_requests: Vec<jx::RegistrationRequest>,
}

fn PendingRegistrationsTable(cx: Scope<PendingRegistrationsTableProps>) -> Element {
    let elections = &cx.props.elections;
    let pending_registration_requests = &cx.props.pending_registration_requests;

    // let is_linking_registration_request_with_election = use_state(cx, || false);

    let link_voter_registration_request_and_election = {
        // TODO: make this work
        // to_owned![is_linking_registration_request_with_election];
        |create_registration_data: jx::CreateRegistrationData| async move {
            // is_linking_registration_request_with_election.set(true);

            let url = get_url("/api/registrations");
            let client = reqwest::Client::new();
            client
                .post(url)
                .json(&create_registration_data)
                .send()
                .await
            // is_linking_registration_request_with_election.set(false);
        }
    };

    render!(
        div {
            rsx!(
                table { class: "table-auto w-full",
                    thead {
                        tr {
                            th { class: "px-4 py-2 text-left", "Voter Name" }
                            th { class: "px-4 py-2 text-left", "Voter CAC ID" }
                            th { class: "px-4 py-2 text-left", "Election Configuration" }
                            th { class: "px-4 py-2 text-left", "Created At" }
                        }
                    }
                    tbody {
                        for registration_request in pending_registration_requests {
                            tr {
                                td { class: "border px-4 py-2", "{registration_request.display_name()}" }
                                td { class: "border px-4 py-2", "{registration_request.common_access_card_id()}" }
                                td {
                                    class: "border px-4 py-2 justify-center",
                                    select {
                                        class: "dark:bg-gray-800 dark:text-white dark:border-gray-600 border-2 rounded-md p-2 focus:outline-none focus:border-blue-500",
                                        oninput: move |event| {
                                            let create_registration_data = serde_json::from_str::<jx::CreateRegistrationData>(event.inner().value.as_str()).expect("parse succeeded");
                                            cx.spawn({
                                                to_owned![link_voter_registration_request_and_election, create_registration_data];
                                                async move {
                                                    log::info!("linking registration request: {create_registration_data:?}");
                                                    match link_voter_registration_request_and_election(create_registration_data).await {
                                                        Ok(response) => {
                                                            if !response.status().is_success() {
                                                                web_sys::window()
                                                                    .unwrap()
                                                                    .alert_with_message(format!("Error linking registration request to election: {}", response.status().as_str()).as_str())
                                                                    .unwrap();
                                                                return;
                                                            }

                                                            log::info!("linked registration request to election: {:?}", response);
                                                        }
                                                        Err(err) => {
                                                            web_sys::window()
                                                                .unwrap()
                                                                .alert_with_message(format!("Error linking registration request to election: {:?}", err).as_str())
                                                                .unwrap();
                                                        }
                                                    }
                                                }
                                            })
                                        },
                                        option {
                                            value: "",
                                            disabled: true,
                                            "Select election configuration"
                                        }
                                        for election in elections.iter() {
                                            optgroup {
                                                label: "{election.title} ({election.election_hash.to_partial()})",
                                                for ballot_style in election.ballot_styles.iter() {
                                                    for precinct_id in ballot_style.precincts.iter() {
                                                        {
                                                            let create_registration_data = jx::CreateRegistrationData {
                                                                election_id: election.id,
                                                                registration_request_id: *registration_request.id(),
                                                                ballot_style_id: ballot_style.id.clone(),
                                                                precinct_id: precinct_id.clone(),
                                                            };
                                                            let value = serde_json::to_string(&create_registration_data)
                                                                .expect("serialization succeeds");
                                                            rsx!(
                                                                option {
                                                                    value: "{value}",
                                                                    "{ballot_style.id} / {precinct_id}"
                                                                }
                                                            )
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                DateOrDateTimeCell {
                                    date_or_datetime: registration_request.created_at(),
                                }
                            }
                        }
                    }
                }
            )
        }
    )
}

#[derive(Debug, PartialEq, Props)]
struct RegistrationsTableProps {
    registrations: Vec<jx::Registration>,
}

fn RegistrationsTable(cx: Scope<RegistrationsTableProps>) -> Element {
    render!(
        table { class: "table-auto w-full",
            thead {
                tr {
                    th { class: "px-4 py-2 text-left", "Voter Name" }
                    th { class: "px-4 py-2 text-left", "Voter CAC ID" }
                    th { class: "px-4 py-2 text-left", "Election Configuration" }
                    th { class: "px-4 py-2 text-left", "Synced" }
                    th { class: "px-4 py-2 text-left", "Created At" }
                }
            }
            tbody {
                for registration in cx.props.registrations.iter() {
                    tr {
                        td { class: "border px-4 py-2", "{registration.display_name()}" }
                        td { class: "border px-4 py-2", "{registration.common_access_card_id()}" }
                        ElectionConfigurationCell {
                            election_title: registration.election_title(),
                            election_hash: registration.election_hash().clone(),
                            precinct_id: registration.precinct_id().clone(),
                            ballot_style_id: registration.ballot_style_id().clone(),
                        }
                        td { class: "border px-4 py-2", if registration.is_synced() { "Yes" } else { "No" } }
                        DateOrDateTimeCell {
                            date_or_datetime: registration.created_at()
                        }
                    }
                }
            }
        }
    )
}
