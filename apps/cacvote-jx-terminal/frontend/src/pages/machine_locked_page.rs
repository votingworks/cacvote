use dioxus::prelude::*;

pub fn MachineLockedPage(cx: Scope) -> Element {
    render!(
        div {
            class: "w-screen h-screen flex flex-col items-center justify-center",
            h1 { class: "text-2xl font-bold mb-4", "CACVote Jurisdiction Terminal Locked" }
            p { class: "text-center", "Please insert an authentication card." }
        }
    )
}
