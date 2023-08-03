use dioxus::prelude::*;

#[derive(Props)]
pub struct ButtonProps<'a> {
    children: Element<'a>,
    disabled: Option<bool>,
    onclick: Option<EventHandler<'a, MouseEvent>>,
}

pub fn Button<'a>(cx: Scope<'a, ButtonProps<'a>>) -> Element {
    render! {
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
    }
}
