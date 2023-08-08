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
    #[redirect("/", || Route::ElectionsPage)]
    #[route("/elections")]
    ElectionsPage,
    #[route("/voters")]
    VotersPage,
}

impl Route {
    fn label(&self) -> &'static str {
        match self {
            Self::ElectionsPage => "Elections",
            Self::VotersPage => "Voters",
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

fn ElectionsPage(cx: Scope) -> Element {
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
                                    DateOrDatetimeColumn {
                                        date_or_datetime: election.date.into(),
                                    }
                                    td { class: "border px-4 py-2", if election.is_synced() { "Yes" } else { "No" } }
                                    DateOrDatetimeColumn {
                                        date_or_datetime: election.created_at.into(),
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

#[derive(PartialEq, Props)]
struct VotersProps<'a> {
    app_data: &'a jx::AppData,
}

fn VotersPage(cx: Scope) -> Element {
    let app_data = use_shared_state::<jx::AppData>(cx).unwrap();
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
            let res = client
                .post(url)
                .json(&create_registration_data)
                .send()
                .await;

            // is_linking_registration_request_with_election.set(false);

            res
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
                                DateOrDatetimeColumn {
                                    date_or_datetime: registration_request.created_at().into(),
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
                        td {
                            class: "border px-4 py-2",
                            p {
                                "{registration.election_title()}"
                                span {
                                    class: "italic text-gray-400",
                                    " ({registration.election_hash().to_partial()})"
                                }
                            }
                            p {
                                "{registration.ballot_style_id()} / {registration.precinct_id()}"
                                span {
                                    class: "italic text-gray-400",
                                    " (Ballot Style / Precinct)"
                                }
                            }
                        }
                        td { class: "border px-4 py-2", if registration.is_synced() { "Yes" } else { "No" } }
                        DateOrDatetimeColumn {
                            date_or_datetime: registration.created_at().into()
                        }
                    }
                }
            }
        }
    )
}

#[inline_props]
fn DateOrDatetimeColumn(cx: Scope, date_or_datetime: DateOrDateTime) -> Element {
    let formatted = format_date(
        date_or_datetime.clone(),
        match date_or_datetime {
            DateOrDateTime::Date(_) => DateFormatOptions::DATE_WITH_DAY_OF_WEEK,
            DateOrDateTime::DateTime(_) => DateFormatOptions::DATETIME,
        },
    );
    render!(
        td {
            class: "border px-4 py-2",
            title: "{date_or_datetime.to_iso8601()}",
            "{formatted}"
        }
    )
}

#[derive(Debug, Clone, PartialEq)]
enum DateOrDateTime {
    Date(time::Date),
    DateTime(time::OffsetDateTime),
}

impl DateOrDateTime {
    fn to_iso8601(&self) -> String {
        match self {
            Self::Date(date) => date.to_string(),
            Self::DateTime(date_time) => date_time.to_string(),
        }
    }
}

impl From<time::Date> for DateOrDateTime {
    fn from(date: time::Date) -> Self {
        Self::Date(date)
    }
}

impl From<time::OffsetDateTime> for DateOrDateTime {
    fn from(date_time: time::OffsetDateTime) -> Self {
        Self::DateTime(date_time)
    }
}

impl From<&time::Date> for DateOrDateTime {
    fn from(date: &time::Date) -> Self {
        Self::Date(*date)
    }
}

impl From<&time::OffsetDateTime> for DateOrDateTime {
    fn from(date_time: &time::OffsetDateTime) -> Self {
        Self::DateTime(*date_time)
    }
}

#[derive(Debug, Clone, Copy, Default)]
enum DateStyle {
    #[default]
    Default,
    Full,
    Long,
    Medium,
    Short,
}

impl DateStyle {
    fn to_js_value(self) -> Option<JsValue> {
        match self {
            Self::Default => None,
            Self::Full => Some(JsValue::from_str("full")),
            Self::Long => Some(JsValue::from_str("long")),
            Self::Medium => Some(JsValue::from_str("medium")),
            Self::Short => Some(JsValue::from_str("short")),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
enum TimeStyle {
    #[default]
    Default,
    Full,
    Long,
    Medium,
    Short,
}

impl TimeStyle {
    fn to_js_value(self) -> Option<JsValue> {
        match self {
            Self::Default => None,
            Self::Full => Some(JsValue::from_str("full")),
            Self::Long => Some(JsValue::from_str("long")),
            Self::Medium => Some(JsValue::from_str("medium")),
            Self::Short => Some(JsValue::from_str("short")),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct DateFormatOptions {
    date_style: DateStyle,
    time_style: TimeStyle,
}

impl DateFormatOptions {
    const DATETIME: Self = Self {
        date_style: DateStyle::Medium,
        time_style: TimeStyle::Short,
    };
    const DATE: Self = Self {
        date_style: DateStyle::Medium,
        time_style: TimeStyle::Default,
    };
    const DATE_WITH_DAY_OF_WEEK: Self = Self {
        date_style: DateStyle::Full,
        time_style: TimeStyle::Default,
    };
}

fn format_date(date_or_datetime: DateOrDateTime, options: DateFormatOptions) -> String {
    let locales = js_sys::Array::new();
    locales.push(&JsValue::from_str("default"));

    let js_options = js_sys::Object::new();

    if let Some(date_style) = options.date_style.to_js_value() {
        let _ = js_sys::Reflect::set(&js_options, &JsValue::from_str("dateStyle"), &date_style);
    }

    if let Some(time_style) = options.time_style.to_js_value() {
        let _ = js_sys::Reflect::set(&js_options, &JsValue::from_str("timeStyle"), &time_style);
    }

    let formatter = js_sys::Intl::DateTimeFormat::new(&locales, &js_options);
    let js_date = js_sys::Date::new_0();

    match date_or_datetime {
        DateOrDateTime::Date(date) => {
            js_date.set_utc_full_year(date.year() as u32);
            js_date.set_utc_month(date.month() as u32);
            js_date.set_utc_date(date.day() as u32);
        }
        DateOrDateTime::DateTime(datetime) => {
            js_date.set_utc_full_year(datetime.year() as u32);
            js_date.set_utc_month(datetime.month() as u32);
            js_date.set_utc_date(datetime.day() as u32);
            js_date.set_utc_hours(datetime.hour() as u32);
            js_date.set_utc_minutes(datetime.minute() as u32);
            js_date.set_utc_seconds(datetime.second() as u32);
        }
    }

    let formatted_date = formatter.format().call1(&formatter, &js_date).unwrap();
    formatted_date.as_string().unwrap()
}
