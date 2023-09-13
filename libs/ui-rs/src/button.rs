use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonSize {
    ExtraLarge,
    Large,
    #[default]
    Medium,
}

#[derive(Props)]
pub struct Props<'a> {
    size: Option<ButtonSize>,
    class: Option<&'a str>,
    children: Element<'a>,
    disabled: Option<bool>,
    onclick: Option<EventHandler<'a, MouseEvent>>,
}

const EXTRA_LARGE_CLASS: &str = "rounded-lg text-xl p-3";
const LARGE_CLASS: &str = "rounded-md text-lg p-2";
const MEDIUM_CLASS: &str = "rounded-md text-base p-1 px-2";

#[allow(non_snake_case)]
pub fn Button<'a>(cx: Scope<'a, Props<'a>>) -> Element {
    let button_size_class = match cx.props.size.unwrap_or_default() {
        ButtonSize::ExtraLarge => EXTRA_LARGE_CLASS,
        ButtonSize::Large => LARGE_CLASS,
        ButtonSize::Medium => MEDIUM_CLASS,
    };
    render! {
       button {
           class: r#"
               bg-purple-500 active:bg-purple-700 disabled:bg-purple-300
               text-white
               {button_size_class}
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
