#![allow(non_snake_case)]

use dioxus::prelude::*;

#[derive(Debug, Props)]
pub struct Props<'a> {
    title: Option<&'a str>,
    children: Element<'a>,
}

pub fn TableCell<'a>(cx: Scope<'a, Props<'a>>) -> Element<'a> {
    render!(td {
        class: "border px-4 py-2 whitespace-nowrap",
        title: cx.props.title,
        &cx.props.children
    })
}
