#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use log::LevelFilter;
use types_rs::rave::jx;
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

#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[layout(Layout)]
    #[redirect("/", || Route::Elections)]
    #[route("/elections")]
    Elections,
    #[route("/voters")]
    Voters,
}

impl Route {
    fn label(&self) -> &'static str {
        match self {
            Self::Elections => "Elections",
            Self::Voters => "Voters",
        }
    }
}

fn App(cx: Scope) -> Element {
    use_shared_state_provider(cx, || jx::AppData::default());
    let app_data = use_shared_state::<jx::AppData>(cx).unwrap();

    use_coroutine(cx, {
        to_owned![app_data];
        |_rx: UnboundedReceiver<i32>| async move {
            let eventsource = web_sys::EventSource::new("/api/status-stream").unwrap();

            let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(data) = event.data().as_string() {
                    match serde_json::from_str::<jx::AppData>(data.as_str()) {
                        Ok(new_app_data) => {
                            log::info!("new app data: {:?}", new_app_data);
                            *app_data.write() = new_app_data;
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

    render!(Router::<Route> {})
}

fn Layout(cx: Scope) -> Element {
    let elections_link_active_class = "bg-gray-300 dark:bg-gray-800";

    render!(
        div {
            class: "h-screen w-screen flex dark:bg-gray-800 dark:text-gray-300",
            div {
                class: "w-1/5 bg-gray-200 dark:bg-gray-700",
                ul {
                    class: "mt-8",
                    for route in [Route::Elections, Route::Voters] {
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
                        class: "fixed bottom-0 w-1/5 bg-gray-300 dark:bg-gray-700 font-bold text-center py-2",
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

fn Elections(cx: Scope) -> Element {
    let app_data = use_shared_state::<jx::AppData>(cx).unwrap();
    let elections = &app_data.read().elections;
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
                if elections.is_empty() {
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
                            for election in elections.iter() {
                                tr {
                                    td {
                                        class: "border px-4 py-2",
                                        title: "Database ID: {election.id}",
                                        "{election.election_hash}"
                                    }
                                    td { class: "border px-4 py-2", "{election.title}" }
                                    td { class: "border px-4 py-2", if election.is_synced() { "Yes" } else { "No" } }
                                    td { class: "border px-4 py-2", "{election.created_at}" }
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
struct VotersProps<'a> {
    app_data: &'a jx::AppData,
}

fn Voters(cx: Scope) -> Element {
    let app_data = use_shared_state::<jx::AppData>(cx).unwrap();
    // let is_linking_registration_request_with_election = use_state(cx, || false);

    let link_voter_registration_request_and_election = {
        // TODO: make this work
        // to_owned![is_linking_registration_request_with_election];
        |create_registration_data: jx::CreateRegistrationData| async move {
            // is_linking_registration_request_with_election.set(true);

            let url = get_url("/api/registrations");
            let client = reqwest::Client::new();
            let res = client
                .post(url)
                .json(&create_registration_data)
                .send()
                .await;

            // is_linking_registration_request_with_election.set(false);

            res
        }
    };

    let app_data = app_data.read();
    let elections = app_data.elections.clone();
    let registration_requests = app_data.registration_requests.clone();
    let registrations = app_data.registrations.clone();
    let pending_registration_requests = registration_requests
        .iter()
        .filter(|registration_request| {
            registrations
                .iter()
                .find(|registration| registration.is_registration_request(registration_request))
                .is_none()
        })
        .map(Clone::clone)
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
                            for registration_request in pending_registration_requests {
                                tr {
                                    td { class: "border px-4 py-2", "{registration_request.display_name()}" }
                                    td { class: "border px-4 py-2", "{registration_request.common_access_card_id()}" }
                                    td {
                                        class: "border px-4 py-2 text-center",
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
                                                "Select election configuration…"
                                            }
                                            for election in elections.iter() {
                                                optgroup {
                                                    label: "{election.title} ({election.election_hash})",
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
                                    td { class: "border px-4 py-2", "{registration_request.created_at()}" }
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
                                    td { class: "border px-4 py-2", "{registration.display_name()}" }
                                    td { class: "border px-4 py-2", "{registration.common_access_card_id()}" }
                                    td { class: "border px-4 py-2", "{registration.election_hash()}" }
                                    td { class: "border px-4 py-2", if registration.is_synced() { "Yes" } else { "No" } }
                                    td { class: "border px-4 py-2", "{registration.created_at()}" }
                                }
                            }
                        }
                    }
                )
            }
        }
    ))
}
