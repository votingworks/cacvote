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

    /// Certificate authority file, used to validate the client certificates
    /// containing a machine's TPM's public key.
    #[arg(long, env = "CA_CERT")]
    pub ca_cert: String,
}

impl Config {
    pub fn load_ca_cert(&self) -> color_eyre::Result<openssl::x509::X509> {
        let ca_cert = std::fs::read(&self.ca_cert)?;
        Ok(openssl::x509::X509::from_pem(&ca_cert)?)
    }
}
