use dioxus::prelude::*;

#[derive(Props)]
pub struct Props<'a> {
    class: Option<&'a str>,
    children: Element<'a>,
    disabled: Option<bool>,
    onclick: Option<EventHandler<'a, MouseEvent>>,
}

#[allow(non_snake_case)]
pub fn Button<'a>(cx: Scope<'a, Props<'a>>) -> Element {
    render! {
       button {
           class: r#"
               rounded-lg text-xl
               bg-purple-500 active:bg-purple-700 disabled:bg-purple-300
               text-white
               p-3 mr-2
               {cx.props.class.unwrap_or_default()}
            "#,
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