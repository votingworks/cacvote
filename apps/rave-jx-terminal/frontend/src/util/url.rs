pub fn get_root_url() -> reqwest::Url {
    let loc = web_sys::window().unwrap().location();
    reqwest::Url::parse(loc.origin().unwrap().as_str()).unwrap()
}

pub fn get_url(path: &str) -> reqwest::Url {
    get_root_url().join(path).unwrap()
}
