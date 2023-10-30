use dioxus::prelude::*;
use dioxus_router::prelude::use_navigator;
use types_rs::rave::client::output::Jurisdiction;

use crate::route::Route;

pub fn ChooseJurisdictionPage(cx: Scope) -> Element {
    let jurisdictions_future = use_future(cx, (), |_| async move {
        #[derive(serde::Deserialize)]
        struct GetJurisdictionsResponse {
            jurisdictions: Vec<Jurisdiction>,
        }
        let url = crate::util::url::get_url("/api/jurisdictions");
        reqwest::get(url)
            .await?
            .json::<GetJurisdictionsResponse>()
            .await
    });

    let nav = use_navigator(cx);

    render!(
        div {
            class: "w-screen h-screen flex flex-col items-center justify-center",
            h1 { class: "text-2xl font-bold mb-4", "Choose Jurisdiction:" }
            match jurisdictions_future.value() {
                Some(Ok(response)) => {
                    rsx! {
                        select {
                            class: "border border-gray-300 rounded-md shadow-sm px-4 py-2",
                            oninput: move |event| {
                                nav.push(Route::ElectionsPage { jurisdiction_id: event.inner().value.clone() });
                            },
                            option { value: "", disabled: true, "Select a jurisdiction" },
                            for jurisdiction in response.jurisdictions.iter() {
                                rsx! { option {
                                    value: "{jurisdiction.id}",
                                    jurisdiction.name.clone()
                                } }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    rsx! {
                        div { "Error loading jurisdictions: {e}" }
                    }
                }
                _ => rsx!{ div { "Loading..." } }
            }
        }
    )
}
