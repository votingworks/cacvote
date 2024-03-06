#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::MessageEvent;

use crate::route::Route;

pub fn AppLayout(cx: Scope) -> Element {
    let nav = use_navigator(cx);

    use_coroutine(cx, {
        to_owned![nav];
        |_rx: UnboundedReceiver<i32>| async move {
            let eventsource = web_sys::EventSource::new("/api/status-stream").unwrap();

            let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(data) = event.data().as_string() {
                    log::info!("received status event: {:?}", data);
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            eventsource.set_onmessage(Some(callback.as_ref().unchecked_ref()));
            callback.forget();
        }
    });

    render!(Outlet::<Route> {})
}
