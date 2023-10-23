use dioxus::prelude::*;
use log::LevelFilter;
use serde::Deserialize;
use types_rs::scan::ScannedBallotStats;
use ui_rs::{Button, DateOrDateTimeCell, TableCell};
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

fn use_scanned_ballot_stats(cx: Scope) -> &UseState<Option<ScannedBallotStats>> {
    let ballot_stats = use_state(cx, || None);

    use_coroutine(cx, {
        to_owned![ballot_stats];
        |_rx: UnboundedReceiver<i32>| async move {
            #[derive(Deserialize)]
            struct StatusStreamEvent {
                stats: ScannedBallotStats,
            }
            let eventsource = web_sys::EventSource::new("/api/status-stream").unwrap();

            let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(data) = event.data().as_string() {
                    match serde_json::from_str::<StatusStreamEvent>(data.as_str()) {
                        Ok(event) => {
                            ballot_stats.set(Some(event.stats));
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

    ballot_stats
}

#[allow(non_snake_case)]
fn App(cx: Scope) -> Element {
    let is_scanning = use_state(cx, || false);
    let ballot_stats = use_scanned_ballot_stats(cx);

    let scan_ballots = move |_| {
        log::info!("scan ballots");
        is_scanning.set(true);

        cx.spawn({
            to_owned![is_scanning];
            async move {
                let client = reqwest::Client::new();
                let result = client.post(get_url("/api/scan")).send().await;
                is_scanning.set(false);

                match result {
                    Ok(response) => {
                        log::info!("response: {:?}", response);

                        if !response.status().is_success() {
                            log::error!("error");
                            return;
                        }

                        log::info!("success");
                    }
                    Err(err) => {
                        log::error!("error: {:?}", err);
                    }
                }
            }
        });
    };

    let is_scanning = *is_scanning.get();
    let ballot_stats = ballot_stats.get();

    render! {
        div {
            class: "h-screen w-screen dark:bg-slate-800",
            div {
                class: "flex-col",
                div {
                    class: "flex flex-row items-center mb-2 pl-2 bg-gray-200 dark:bg-gray-700",
                    div {
                        class: "text-3xl flex-grow font-bold text-gray-700 dark:text-gray-200",
                        "RAVE Scan"
                    }
                    div {
                        class: "m-2",
                        Button {
                            onclick: scan_ballots,
                            disabled: is_scanning,
                            if is_scanning {
                                "Scanning…"
                            } else {
                                "Scan Ballots"
                            },
                        }
                    }
                }
                div {
                    class: "text-xl text-center text-gray-400 dark:text-gray-300 px-3",
                    if let Some(ballot_stats) = ballot_stats {
                        rsx! {
                            h3 {
                                class: "text-left",
                                "Batches"
                            }
                            table {
                                class: "text-sm",
                                thead {
                                    tr {
                                        th { "ID" }
                                        th { "Status" }
                                        th { "Started At" }
                                        th { "Ended At" }
                                    }
                                }
                                tbody {
                                    for batch in ballot_stats.batches.clone().into_iter() {
                                        rsx! {
                                            tr {
                                                TableCell { batch.id.to_string() }
                                                TableCell {
                                                    match (batch.ballot_count, batch.election_count, batch.synced_count) {
                                                        (0, _, _) => "No ballots".to_owned(),
                                                        (1, _, 0) => "1 ballot (pending sync)".to_owned(),
                                                        (1, _, 1) => "1 ballot (synced)".to_owned(),
                                                        (b, 1, s) if b == s => format!("{b} ballots, 1 election (synced)"),
                                                        (b, 1, s) => format!("{b} ballots, 1 election ({s} synced)"),
                                                        (b, e, s) if b == s => format!("{b} ballots, {e} elections (synced)"),
                                                        (b, e, s) => format!("{b} ballots, {e} elections ({s} synced)"),
                                                    }
                                                }
                                                DateOrDateTimeCell {
                                                    date_or_datetime: batch.started_at,
                                                }
                                                match batch.ended_at {
                                                    Some(ended_at) => {
                                                        rsx! {
                                                            DateOrDateTimeCell { date_or_datetime: ended_at }
                                                        }
                                                    }
                                                    None => {
                                                        rsx! {
                                                            TableCell {
                                                                "Scanning…"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
       }
    }
}
