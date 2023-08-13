const TEN_MB: usize = 10 * 1024 * 1024;

pub const DEFAULT_PORT: u16 = 8000;
pub const MAX_REQUEST_SIZE: usize = TEN_MB;

/// Checks that all required configuration is present.
///
/// # Panics
///
/// This function will panic if any required configuration is not present.
pub(crate) fn check() {
    let _ = *DATABASE_URL;
    let _ = *PORT;
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
