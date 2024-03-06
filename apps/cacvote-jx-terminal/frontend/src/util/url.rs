#[allow(dead_code)]
pub fn get_root_url() -> reqwest::Url {
    let loc = web_sys::window().unwrap().location();
    reqwest::Url::parse(loc.origin().unwrap().as_str()).unwrap()
}

#[allow(dead_code)]
pub fn get_url<'a, S: Into<&'a str>>(path: S) -> reqwest::Url {
    get_root_url().join(path.into()).unwrap()
}
