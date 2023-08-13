use std::time::Duration;

const TEN_MB: usize = 10 * 1024 * 1024;

pub const DEFAULT_PORT: u16 = 5001;
pub const MAX_REQUEST_SIZE: usize = TEN_MB;
pub const SYNC_INTERVAL: Duration = Duration::from_secs(5);

/// Checks that all required configuration is present.
///
/// # Panics
///
/// This function will panic if any required configuration is not present.
pub(crate) fn check() {
    let _ = *RAVE_URL;
    let _ = *DATABASE_URL;
    let _ = *VX_MACHINE_ID;
    let _ = *PORT;
}

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

lazy_static::lazy_static! {
    pub static ref DATABASE_URL: String = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
}

lazy_static::lazy_static! {
    pub static ref PORT: u16 =
        match std::env::var("PORT").ok() {
            Some(port) => port.parse().expect("PORT must be a valid port number"),
            None => DEFAULT_PORT,
        };
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

impl std::fmt::Display for PORT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(*self).to_string())
    }
}
