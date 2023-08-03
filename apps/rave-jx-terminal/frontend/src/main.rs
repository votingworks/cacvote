#![allow(non_snake_case)]

use std::sync::Arc;

use dioxus::prelude::*;
use log::LevelFilter;
use ui_rs::FileButton;

fn main() {
    // Init debug
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    console_error_panic_hook::set_once();

    log::info!("starting app");
    dioxus_web::launch(App);
}

fn get_root_url() -> reqwest::Url {
    let loc = web_sys::window().unwrap().location();
    reqwest::Url::parse(loc.origin().unwrap().as_str()).unwrap()
}

fn get_url(path: &str) -> reqwest::Url {
    get_root_url().join(path).unwrap()
}

async fn read_file_as_bytes(file_engine: Arc<dyn FileEngine>) -> Option<Vec<u8>> {
    let files = file_engine.files();
    let file = files.first()?;
    file_engine.read_file(&file).await
}

fn App(cx: Scope) -> Element {
    let is_uploading = use_state(cx, || false);

    let upload_election = {
        to_owned![is_uploading];
        |election_data: Vec<u8>| async move {
            is_uploading.set(true);

            let url = get_url("/api/elections");
            let client = reqwest::Client::new();
            let res = client
                .post(url)
                .body(election_data)
                .header("Content-Type", "application/json")
                .send()
                .await;

            is_uploading.set(false);

            res
        }
    };

    cx.render(rsx! (
        div {
           class: "h-screen w-screen flex justify-center items-center dark:bg-slate-800",
           FileButton {
                "Import Election",
                onfile: move |file_engine: Arc<dyn FileEngine>| {
                    cx.spawn({
                        to_owned![upload_election, file_engine];
                        async move {
                            if let Some(election_data) = read_file_as_bytes(file_engine).await {
                                match upload_election(election_data).await {
                                    Ok(response) => {
                                        if !response.status().is_success() {
                                            web_sys::window()
                                                .unwrap()
                                                .alert_with_message(format!("Error uploading election: {}", response.status().as_str()).as_str())
                                                .unwrap();
                                            return;
                                        }

                                        web_sys::window()
                                            .unwrap()
                                            .alert_with_message("Election uploaded.")
                                            .unwrap();
                                    }
                                    Err(err) => {
                                        web_sys::window()
                                            .unwrap()
                                            .alert_with_message(format!("Error uploading election: {:?}", err).as_str())
                                            .unwrap();
                                    }
                                }
                            };
                        }
                    });
                },
           }
        }
    ))
}
