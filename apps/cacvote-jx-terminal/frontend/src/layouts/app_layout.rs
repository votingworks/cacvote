#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use types_rs::cacvote::SessionData;
use wasm_bindgen::prelude::*;
use web_sys::MessageEvent;

use crate::route::Route;

pub fn AppLayout(cx: Scope) -> Element {
    use_shared_state_provider(cx, SessionData::default);
    let session_data = use_shared_state::<SessionData>(cx).unwrap();
    let nav = use_navigator(cx);

    use_coroutine(cx, {
        to_owned![nav, session_data];
        |_rx: UnboundedReceiver<i32>| async move {
            log::info!("starting status stream");
            let eventsource = web_sys::EventSource::new("/api/status-stream").unwrap();

            let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(data) = event.data().as_string() {
                    log::info!("received status event: {:?}", data);
                    match serde_json::from_str::<SessionData>(&data) {
                        Ok(new_session_data) => {
                            log::info!("updating session data: {:?}", new_session_data);

                            match new_session_data {
                                SessionData::Authenticated { .. } => {
                                    log::info!("redirecting to elections page");
                                    nav.push(Route::ElectionsPage);
                                }
                                SessionData::Unauthenticated { .. } => {
                                    log::info!("redirecting to machine locked page");
                                    nav.push(Route::MachineLockedPage);
                                }
                            }

                            *session_data.write() = new_session_data;
                        }
                        Err(err) => {
                            log::error!("failed to parse session data: {:?}", err);
                        }
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            eventsource.set_onmessage(Some(callback.as_ref().unchecked_ref()));
            callback.forget();
        }
    });

    render!(Outlet::<Route> {})
}
