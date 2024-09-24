//! Application configuration.

use std::path::{Path, PathBuf};

use clap::Parser;
use color_eyre::eyre::Context;

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

    /// Certificate authority certificate files, used to validate the client
    /// certificates containing a CAC's public key.
    #[arg(long, env = "CAC_CA_CERTS", value_delimiter = ',')]
    pub cac_ca_certs: Vec<PathBuf>,

    /// Certificate authority certificate files, used to validate the client
    /// certificates containing a machine's TPM's public key.
    #[arg(long, env = "MACHINE_CA_CERT")]
    pub machine_ca_cert: PathBuf,
}

impl Config {
    pub fn cac_ca_store(&self) -> color_eyre::Result<openssl::x509::store::X509Store> {
        dbg!(&self.cac_ca_certs);
        let mut builder = openssl::x509::store::X509StoreBuilder::new()?;

        for ca_cert in &self.cac_ca_certs {
            builder.add_cert(load_cert(ca_cert)?)?;
        }

        Ok(builder.build())
    }

    pub fn machine_ca_cert(&self) -> color_eyre::Result<openssl::x509::X509> {
        load_cert(&self.machine_ca_cert)
    }
}

fn load_cert<P>(path: P) -> color_eyre::Result<openssl::x509::X509>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let ca_cert = std::fs::read(path)
        .with_context(|| format!("failed to read CA certificate at {path:?}"))?;
    Ok(openssl::x509::X509::from_pem(&ca_cert)
        .or_else(|_| openssl::x509::X509::from_der(&ca_cert))?)
}
