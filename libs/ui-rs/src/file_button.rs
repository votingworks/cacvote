use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine};
use dioxus::prelude::*;
use wasm_bindgen::JsCast;

use crate::button::Button;

#[derive(Props)]
pub struct Props<'a> {
    class: Option<&'a str>,
    children: Element<'a>,
    disabled: Option<bool>,
    #[props(into)]
    onfile: Option<EventHandler<'a, Arc<dyn FileEngine>>>,
}

/// A button that opens a file dialog when clicked.
///
/// # Example
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use dioxus::prelude::*;
/// use ui_rs::FileButton;
///
/// fn app(cx: Scope) -> Element {
///     render! {
///         FileButton {
///             "Click me",
///             onfile: {
///                 move |file_engine: Arc<dyn FileEngine>| {
///                     // read the first file, assuming there is at least one
///                     cx.spawn(async move {
///                         let files = file_engine.files();
///                         let file = files.first().unwrap();
///                         let content = file_engine.read_file(&file).await.unwrap();
///                         log::info!("file content: {:?}", content);
///                     });
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// # Panics
///
/// This component may panic if the `window.crypto` API is not available.
/// ```
#[allow(non_snake_case)]
pub fn FileButton<'a>(cx: Scope<'a, Props<'a>>) -> Element {
    let id = use_state(cx, || {
        let mut randombytes = [0u8; 32];
        getrandom::getrandom(&mut randombytes).unwrap();
        STANDARD.encode(randombytes.as_ref())
    });

    render! {
        input {
            id: id.as_ref(),
            r#type: "file",
            class: "hidden",
            disabled: cx.props.disabled,
            oninput: {
                |e| {
                    if let Some(onfile) = &cx.props.onfile {
                        if let Some(file_engine) = &e.data.files {
                            onfile.call(file_engine.clone());
                        }
                    }
                }
            },
        }
        Button {
            class: cx.props.class.unwrap_or(""),
            disabled: cx.props.disabled.unwrap_or(false),
            &cx.props.children
            onclick: {
                to_owned![id];
                move |_e: MouseEvent| {
                    let input = web_sys::window().unwrap().document().unwrap().get_element_by_id(&id).unwrap();
                    input.dyn_ref::<web_sys::HtmlInputElement>().unwrap().click();
                }
            }
        }
    }
}
