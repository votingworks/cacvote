use dioxus::prelude::*;

pub fn VotersPage(cx: Scope) -> Element {
    render!(
        h1 { class: "text-2xl font-bold mb-4", "Pending Registrations" }
        rsx!("No pending registrations")

        h1 { class: "text-2xl font-bold mt-4 mb-4", "Registrations" }
        rsx!("No registrations")
    )
}
