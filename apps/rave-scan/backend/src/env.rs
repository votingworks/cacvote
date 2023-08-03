lazy_static::lazy_static! {
    pub static ref RAVE_URL: reqwest::Url = reqwest::Url::parse(
        std::env::var("RAVE_URL")
            .expect("RAVE_URL must be set")
            .as_str(),
    )
    .expect("RAVE_URL must be a valid URL");
}

lazy_static::lazy_static! {
    pub static ref VX_MACHINE_ID: String = std::env::var("VX_MACHINE_ID")
        .expect("VX_MACHINE_ID must be set");
}

impl PartialEq<String> for VX_MACHINE_ID {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&str> for VX_MACHINE_ID {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<VX_MACHINE_ID> for String {
    fn eq(&self, other: &VX_MACHINE_ID) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<VX_MACHINE_ID> for &str {
    fn eq(&self, other: &VX_MACHINE_ID) -> bool {
        *self == other.as_str()
    }
}
