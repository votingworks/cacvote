use std::rc::Rc;

use dioxus::prelude::*;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::MessageEvent;

fn main() {
    // Init debug
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    console_error_panic_hook::set_once();

    log::info!("starting app");
    dioxus_web::launch(App);
}

#[derive(Props)]
struct ButtonProps<'a> {
    children: Element<'a>,
    disabled: Option<bool>,
    onclick: Option<EventHandler<'a, MouseEvent>>,
}

fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element {
    render!(
        button {
            class: "rounded-lg text-3xl bg-purple-500 active:bg-purple-700 disabled:bg-purple-300 text-white p-5 mr-2",
            disabled: cx.props.disabled,
            onclick: |e| {
                if let Some(onclick) = &cx.props.onclick {
                    onclick.call(e);
                }
            },
            &cx.props.children
        }
    )
}

fn get_root_url() -> reqwest::Url {
    let loc = web_sys::window().unwrap().location();
    reqwest::Url::parse(loc.origin().unwrap().as_str()).unwrap()
}

fn get_url(path: &str) -> reqwest::Url {
    get_root_url().join(path).unwrap()
}

#[derive(Debug, Deserialize, PartialEq, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScannedBallotStats {
    pub total: i64,
    pub pending: i64,
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
           class: "h-screen w-screen flex justify-center items-center dark:bg-slate-800",
           div {
               class: "flex-col",
               div {
                   class: "flex-row mb-2",
                   Button {
                       if is_scanning {
                           "Scanning…"
                       } else {
                           "Scan Ballots"
                       },
                       onclick: scan_ballots,
                       disabled: is_scanning,
                   }
               }
               div {
                   class: "text-xl text-center text-gray-400 dark:text-gray-300",
                   if let Some(ballot_stats) = ballot_stats {
                       rsx! {
                           span { "{ballot_stats.total} scanned ballot(s)" }
                           span {
                               if ballot_stats.pending == 0 {
                                   rsx!(", synced to server")
                               } else {
                                   rsx!(", {ballot_stats.pending} pending sync")
                               }
                           }
                       }
                   }
               }
           }
       }
    }
}
