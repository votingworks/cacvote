#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use types_rs::rave::jx;
use wasm_bindgen::prelude::*;
use web_sys::MessageEvent;

use crate::route::Route;

#[derive(PartialEq, Props)]
pub struct LoggedInLayoutProps {
    jurisdiction_id: String,
}

pub fn LoggedInLayout(cx: Scope<LoggedInLayoutProps>) -> Element {
    use_shared_state_provider(cx, jx::LoggedInAppData::default);
    let app_data = use_shared_state::<jx::LoggedInAppData>(cx).unwrap();
    let jurisdiction_id = &cx.props.jurisdiction_id;

    use_coroutine(cx, {
        to_owned![app_data, jurisdiction_id];
        |_rx: UnboundedReceiver<i32>| async move {
            let eventsource = web_sys::EventSource::new(
                format!("/api/status-stream?jurisdiction_id={jurisdiction_id}").as_str(),
            )
            .unwrap();

            let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(data) = event.data().as_string() {
                    match serde_json::from_str::<jx::LoggedInAppData>(data.as_str()) {
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

    let jurisdiction_id = &cx.props.jurisdiction_id;

    render!(
        div {
            class: "h-screen w-screen flex dark:bg-gray-800 dark:text-gray-300",
            div {
                class: "w-1/5 bg-gray-200 dark:bg-gray-700",
                ul {
                    class: "mt-8",
                    for route in [
                        Route::ElectionsPage { jurisdiction_id: jurisdiction_id.clone() },
                        Route::VotersPage { jurisdiction_id: jurisdiction_id.clone() },
                        Route::BallotsPage { jurisdiction_id: jurisdiction_id.clone() }
                    ] {
                        li {
                            Link {
                                to: route.clone(),
                                active_class: "bg-gray-300 dark:bg-gray-800",
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
