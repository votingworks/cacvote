#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use types_rs::cacvote::jx::{self, AppData};
use wasm_bindgen::prelude::*;
use web_sys::MessageEvent;

use crate::route::Route;

pub fn AppLayout(cx: Scope) -> Element {
    use_shared_state_provider(cx, jx::AppData::default);
    let app_data = use_shared_state::<jx::AppData>(cx).unwrap();
    let nav = use_navigator(cx);

    use_coroutine(cx, {
        to_owned![app_data, nav];
        |_rx: UnboundedReceiver<i32>| async move {
            let eventsource = web_sys::EventSource::new("/api/status-stream").unwrap();

            let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(data) = event.data().as_string() {
                    match serde_json::from_str::<jx::AppData>(data.as_str()) {
                        Ok(new_app_data) => {
                            log::info!("new app data: {:?}", new_app_data);

                            if matches!(new_app_data, AppData::LoggedOut { .. }) {
                                nav.push(Route::MachineLockedPage);
                            }

                            if matches!(
                                (&*app_data.read(), &new_app_data),
                                (AppData::LoggedOut { .. }, AppData::LoggedIn { .. })
                            ) {
                                nav.push(Route::ElectionsPage);
                            }

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

    render!(Outlet::<Route> {})
}
