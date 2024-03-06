use dioxus::prelude::*;

pub fn BallotsPage(cx: Scope) -> Element {
    render!(
        h1 { class: "text-2xl font-bold mb-4", "Printed Ballots" }
        rsx!("No printed ballots")
    )
}
