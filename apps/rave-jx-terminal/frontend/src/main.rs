#![allow(non_snake_case)]

use std::sync::Arc;

use chrono::Utc;
use dioxus::{html::link, prelude::*};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use ui_rs::FileButton;
use wasm_bindgen::prelude::*;
use web_sys::MessageEvent;

fn main() {
    // Init debug
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    console_error_panic_hook::set_once();

    log::info!("starting app");
    dioxus_web::launch(App);
}

fn get_root_url() -> reqwest::Url {
    let loc = web_sys::window().unwrap().location();
    reqwest::Url::parse(loc.origin().unwrap().as_str()).unwrap()
}

fn get_url(path: &str) -> reqwest::Url {
    get_root_url().join(path).unwrap()
}

async fn read_file_as_bytes(file_engine: Arc<dyn FileEngine>) -> Option<Vec<u8>> {
    let files = file_engine.files();
    let file = files.first()?;
    file_engine.read_file(&file).await
}

#[derive(Debug)]
enum Pages {
    Elections,
    Voters,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Election {
    title: String,
}

#[derive(Debug, PartialEq)]
struct ElectionDefinition {
    election: Election,
    election_data: String,
}

impl<'de> Deserialize<'de> for ElectionDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let election_data = String::deserialize(deserializer)?;
        Ok(Self {
            election: serde_json::from_str(&election_data).map_err(serde::de::Error::custom)?,
            election_data,
        })
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[repr(transparent)]
struct ElectionHash(String);

impl std::fmt::Display for ElectionHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0[..8])
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ElectionRecord {
    pub id: String,
    pub server_id: Option<String>,
    pub client_id: String,
    pub machine_id: String,
    pub definition: ElectionDefinition,
    pub election_hash: ElectionHash,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ElectionRecord {
    fn title(&self) -> &str {
        &self.definition.election.title
    }

    fn relative_time(&self) -> chrono_humanize::HumanTime {
        self.created_at.signed_duration_since(Utc::now()).into()
    }

    fn synced(&self) -> bool {
        self.server_id.is_some()
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RegistrationRequest {
    id: String,
    server_id: String,
    client_id: String,
    machine_id: String,
    common_access_card_id: String,
    given_name: String,
    family_name: String,
    address_line_1: String,
    address_line_2: Option<String>,
    city: String,
    state: String,
    postal_code: String,
    state_id: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl RegistrationRequest {
    fn full_name(&self) -> String {
        format!("{} {}", self.given_name, self.family_name)
    }

    fn relative_time(&self) -> chrono_humanize::HumanTime {
        self.created_at.signed_duration_since(Utc::now()).into()
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Registration {
    pub id: String,
    pub server_id: Option<String>,
    pub client_id: String,
    pub machine_id: String,
    pub common_access_card_id: String,
    pub registration_request_id: String,
    pub election_id: String,
    pub precinct_id: String,
    pub ballot_style_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Registration {
    fn full_name(&self) -> String {
        self.registration_request_id.to_string()
    }

    fn relative_time(&self) -> chrono_humanize::HumanTime {
        self.created_at.signed_duration_since(Utc::now()).into()
    }

    fn synced(&self) -> bool {
        self.server_id.is_some()
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct AppData {
    elections: Vec<ElectionRecord>,
    registration_requests: Vec<RegistrationRequest>,
    registrations: Vec<Registration>,
}

fn App(cx: Scope) -> Element {
    let active_page = use_state(cx, || Pages::Elections);

    let app_data = use_state(cx, || AppData::default());

    use_coroutine(cx, {
        to_owned![app_data];
        |_rx: UnboundedReceiver<i32>| async move {
            let eventsource = web_sys::EventSource::new("/api/status-stream").unwrap();

            let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(data) = event.data().as_string() {
                    match serde_json::from_str::<AppData>(data.as_str()) {
                        Ok(new_app_data) => {
                            log::info!("new app data: {:?}", new_app_data);
                            app_data.set(new_app_data);
                        }
                        Err(err) => {
                            log::error!("error deserializing status event: {:?}", err);
                        }
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            eventsource.set_onmessage(Some(callback.as_ref().unchecked_ref()));
            callback.forget();
        }
    });

    let elections_link_active_class = match active_page.get() {
        Pages::Elections => "bg-gray-300",
        _ => "",
    };
    let voters_link_active_class = match active_page.get() {
        Pages::Voters => "bg-gray-300",
        _ => "",
    };

    cx.render(rsx! (
        div {
            class: "h-screen w-screen flex dark:bg-slate-800",
            div {
                class: "w-1/5 bg-gray-200",
                ul {
                    class: "mt-8",
                    li {
                        class: "px-4 py-2 hover:bg-gray-300 hover:cursor-pointer {elections_link_active_class}",
                        onclick: {
                            to_owned![active_page];
                            move |_| active_page.set(Pages::Elections)
                        },
                        "Elections"
                    }
                    li {
                        class: "px-4 py-2 hover:bg-gray-300 hover:cursor-pointer {voters_link_active_class}",
                        onclick: {
                            to_owned![active_page];
                            move |_| active_page.set(Pages::Voters)
                        },
                        "Voters"
                    }
                    li {
                        class: "fixed bottom-0 w-1/5 bg-gray-300 font-bold text-center py-2",
                        "RAVE Scan"
                    }
                }
            }
            div { class: "w-4/5 p-8",
                match active_page.get() {
                    Pages::Elections =>
                        rsx!(ElectionsPage {
                            elections: &app_data.get().elections,
                        }),
                    Pages::Voters =>
                        rsx!(VotersPage {
                            elections: &app_data.get().elections,
                            registration_requests: &app_data.get().registration_requests,
                            registrations: &app_data.get().registrations,
                        }),
                    }
            }
        }
    ))
}

#[derive(PartialEq, Props)]
struct ElectionsPageProps<'a> {
    elections: &'a Vec<ElectionRecord>,
}

fn ElectionsPage<'a>(cx: Scope<'a, ElectionsPageProps>) -> Element<'a> {
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

    cx.render(rsx! (
        div {
                h1 { class: "text-2xl font-bold mb-4", "Elections" }
                if cx.props.elections.is_empty() {
                    rsx!(div { "No elections found." })
                } else {
                    rsx!(table { class: "table-auto w-full",
                        thead {
                            tr {
                                th { class: "px-4 py-2 text-left", "Election ID" }
                                th { class: "px-4 py-2 text-left", "Title" }
                                th { class: "px-4 py-2 text-left", "Synced" }
                                th { class: "px-4 py-2 text-left", "Created At" }
                            }
                        }
                        tbody {
                            for election in cx.props.elections.iter() {
                                tr {
                                    td {
                                        class: "border px-4 py-2",
                                        title: "Database ID: {election.id}",
                                        election.election_hash.to_string()
                                    }
                                    td { class: "border px-4 py-2", "{election.title()}" }
                                    td { class: "border px-4 py-2", if election.synced() { "Yes" } else { "No" } }
                                    td { class: "border px-4 py-2", "{election.relative_time()}" }
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
    ))
}

#[derive(PartialEq, Props)]
struct VotersPageProps<'a> {
    elections: &'a Vec<ElectionRecord>,
    registration_requests: &'a Vec<RegistrationRequest>,
    registrations: &'a Vec<Registration>,
}

fn VotersPage<'a>(cx: Scope<'a, VotersPageProps>) -> Element<'a> {
    // let is_linking_registration_request_with_election = use_state(cx, || false);

    let link_voter_registration_request_and_election = {
        // TODO: make this work
        // to_owned![is_linking_registration_request_with_election];
        |election_id: String, registration_request_id: String| async move {
            #[derive(Serialize)]
            #[serde(rename_all = "camelCase")]
            struct LinkVoterRegistrationRequestAndElectionRequest {
                election_id: String,
                registration_request_id: String,
            }

            // is_linking_registration_request_with_election.set(true);

            let url = get_url("/api/registrations");
            let client = reqwest::Client::new();
            let res = client
                .post(url)
                .json(&LinkVoterRegistrationRequestAndElectionRequest {
                    election_id,
                    registration_request_id,
                })
                .send()
                .await;

            // is_linking_registration_request_with_election.set(false);

            res
        }
    };

    let elections = cx.props.elections;
    let registration_requests = cx.props.registration_requests;
    let registrations = cx.props.registrations;
    let pending_registration_requests = registration_requests
        .iter()
        .filter(|registration_request| {
            registrations
                .iter()
                .find(|registration| {
                    registration.registration_request_id == registration_request.id
                })
                .is_none()
        })
        .collect::<Vec<_>>();

    cx.render(rsx! (
        div {
            h1 { class: "text-2xl font-bold mb-4", "Pending Registrations" }
            if pending_registration_requests.is_empty() {
                rsx!("No pending registrations")
            } else {
                rsx!(
                    table { class: "table-auto w-full",
                        thead {
                            tr {
                                th { class: "px-4 py-2 text-left", "Voter Name" }
                                th { class: "px-4 py-2 text-left", "Voter CAC ID" }
                                th { class: "px-4 py-2 text-left", "Election" }
                                th { class: "px-4 py-2 text-left", "Created At" }
                            }
                        }
                        tbody {
                            for registration_request in registration_requests.iter().filter(|registration_request| registrations.iter().find(|registration| registration.registration_request_id == registration_request.id).is_none()) {
                                tr {
                                    td { class: "border px-4 py-2", "{registration_request.full_name()}" }
                                    td { class: "border px-4 py-2", "{registration_request.common_access_card_id}" }
                                    td {
                                        class: "border px-4 py-2 text-center",
                                        select {
                                            oninput: move |event| {
                                                let election_id = &event.inner().value;
                                                let registration_request_id = &registration_request.id;
                                                cx.spawn({
                                                    to_owned![link_voter_registration_request_and_election, election_id, registration_request_id];
                                                    async move {
                                                        log::info!("linking registration request {} to election {}", registration_request_id, election_id);
                                                        match link_voter_registration_request_and_election(election_id.to_string(), registration_request_id.to_string()).await {
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
                                                "Link voter with an election"
                                            }
                                            for election in elections.iter() {
                                                option {
                                                    value: "{election.id}",
                                                    "{election.title()} ({election.election_hash.to_string()})"
                                                }
                                            }
                                        }
                                    }
                                    td { class: "border px-4 py-2", "{registration_request.relative_time()}" }
                                }
                            }
                        }
                    }
                )
            }

            h1 { class: "text-2xl font-bold mt-4 mb-4", "Registrations" }
            if registrations.is_empty() {
                rsx!("No registrations")
            } else {
                rsx!(
                    table { class: "table-auto w-full",
                        thead {
                            tr {
                                th { class: "px-4 py-2 text-left", "Voter Name" }
                                th { class: "px-4 py-2 text-left", "Voter CAC ID" }
                                th { class: "px-4 py-2 text-left", "Election ID" }
                                th { class: "px-4 py-2 text-left", "Synced" }
                                th { class: "px-4 py-2 text-left", "Created At" }
                            }
                        }
                        tbody {
                            for registration in registrations.iter() {
                                tr {
                                    td { class: "border px-4 py-2", "{registration.full_name()}" }
                                    td { class: "border px-4 py-2", "{registration.common_access_card_id}" }
                                    td { class: "border px-4 py-2", "TODO" }
                                    td { class: "border px-4 py-2", if registration.synced() { "Yes" } else { "No" } }
                                    td { class: "border px-4 py-2", "{registration.relative_time()}" }
                                }
                            }
                        }
                    }
                )
            }
        }
    ))
}
