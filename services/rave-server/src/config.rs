//! Application configuration.

use clap::Parser;

const TEN_MB: usize = 10 * 1024 * 1024;

pub(crate) const MAX_REQUEST_SIZE: usize = TEN_MB;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub(crate) struct Config {
    /// URL of the PostgreSQL database, e.g. `postgres://user:pass@host:port/dbname`.
    #[arg(long, env = "DATABASE_URL")]
    pub(crate) database_url: String,

    /// Port to listen on.
    #[arg(long, env = "PORT")]
    pub(crate) port: u16,

    /// Log level.
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    pub(crate) log_level: tracing::Level,

    /// Optional path to an election definition to load on startup.
    #[arg(long, env = "ELECTION_DEFINITION_PATH")]
    pub(crate) election_definition_path: Option<String>,

    /// Automatically link pending registration requests with the latest election.
    #[arg(
        long,
        env = "AUTOMATICALLY_LINK_PENDING_REGISTRATION_REQUESTS_WITH_LATEST_ELECTION"
    )]
    pub(crate) automatically_link_pending_registration_requests_with_latest_election: bool,
}
