const TEN_MB: usize = 10 * 1024 * 1024;

pub const DEFAULT_PORT: u16 = 8000;
pub const MAX_REQUEST_SIZE: usize = TEN_MB;

lazy_static::lazy_static! {
    pub static ref PORT: u16 =
        match std::env::var("PORT").ok() {
            Some(port) => port.parse().expect("PORT must be a valid port number"),
            None => DEFAULT_PORT,
        };
}
