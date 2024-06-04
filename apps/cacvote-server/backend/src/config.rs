//! Application configuration.

use clap::Parser;

const TEN_MB: usize = 10 * 1024 * 1024;

pub const MAX_REQUEST_SIZE: usize = TEN_MB;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub struct Config {
    /// URL of the PostgreSQL database, e.g. `postgres://user:pass@host:port/dbname`.
    #[arg(long, env = "DATABASE_URL")]
    pub database_url: String,

    /// Port to listen on.
    #[arg(long, env = "PORT")]
    pub port: u16,

    /// Log level.
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    pub log_level: tracing::Level,
}
