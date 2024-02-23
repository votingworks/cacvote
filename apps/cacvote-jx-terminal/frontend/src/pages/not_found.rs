#![allow(non_snake_case)]

use dioxus::prelude::*;

#[derive(PartialEq, Props)]
pub struct NotFoundProps {
    pub segments: Vec<String>,
}

pub fn NotFound(cx: Scope<NotFoundProps>) -> Element {
    let segments = cx.props.segments.join("/");
    render!(
        div {
            class: "w-screen h-screen flex flex-col items-center justify-center",
            h1 { class: "text-2xl font-bold mb-4", "404 Not Found" }
            p { class: "text-center", "The page {segments} was not found." }
        }
    )
}
