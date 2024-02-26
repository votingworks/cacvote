//! Application configuration.

use std::time::Duration;

use clap::Parser;

const TEN_MB: usize = 10 * 1024 * 1024;

pub(crate) const MAX_REQUEST_SIZE: usize = TEN_MB;
pub(crate) const SYNC_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub(crate) struct Config {
    /// URL of the CACVote server, e.g. `https://cacvote.example.com/`.
    #[arg(long, env = "CACVOTE_URL")]
    pub(crate) cacvote_url: reqwest::Url,

    /// URL of the PostgreSQL database, e.g. `postgres://user:pass@host:port/dbname`.
    #[arg(long, env = "DATABASE_URL")]
    pub(crate) database_url: String,

    /// ID of this machine, e.g. `machine-1`.
    #[arg(long, env = "VX_MACHINE_ID")]
    pub(crate) machine_id: String,

    /// Port to listen on.
    #[arg(long, env = "PORT")]
    pub(crate) port: u16,

    /// Directory to serve static files from.
    #[arg(long, env = "PUBLIC_DIR")]
    pub(crate) public_dir: Option<std::path::PathBuf>,

    /// Log level.
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    pub(crate) log_level: tracing::Level,
}
