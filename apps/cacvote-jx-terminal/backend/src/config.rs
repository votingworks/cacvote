//! Application configuration.

use std::{path::PathBuf, time::Duration};

use auth_rs::{card_details::extract_field_value, certs::VX_CUSTOM_CERT_FIELD_JURISDICTION};
use cacvote_server_client::signer;
use clap::Parser;
use color_eyre::eyre::{bail, Context};
use openssl::x509::X509;
use types_rs::cacvote::JurisdictionCode;

const TEN_MB: usize = 10 * 1024 * 1024;

pub(crate) const MAX_REQUEST_SIZE: usize = TEN_MB;
pub(crate) const SYNC_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
pub(crate) struct Config {
    /// URL of the CACvote server, e.g. `https://cacvote.example.com/`.
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

    /// Certificate authority PEM file.
    #[arg(long, env = "CA_CERT")]
    pub(crate) ca_cert: PathBuf,

    /// Signer to use for signing server request payloads.
    #[arg(long, env = "SIGNER")]
    pub(crate) signer: signer::Description,

    /// Directory to serve static files from.
    #[arg(long, env = "PUBLIC_DIR")]
    pub(crate) public_dir: Option<PathBuf>,

    /// Log level.
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    pub(crate) log_level: tracing::Level,

    /// ElectionGuard Java CLI CLASSPATH.
    #[arg(long, env = "EG_CLASSPATH")]
    pub(crate) eg_classpath: PathBuf,
}

impl Config {
    pub(crate) fn ca_cert(&self) -> color_eyre::Result<X509> {
        openssl::x509::X509::from_pem(
            &std::fs::read(&self.ca_cert).context("CA_CERT cannot be read")?,
        )
        .context("CA_CERT is not a valid certificate")
    }

    /// Returns the jurisdiction code from the CA certificate.
    pub(crate) fn jurisdiction_code(&self) -> color_eyre::Result<JurisdictionCode> {
        let raw_jurisdiction_code = match extract_field_value(&self.ca_cert()?, VX_CUSTOM_CERT_FIELD_JURISDICTION)
                .context("Unable to extract jurisdiction code from CA_CERT")? {
                    Some(value) => value,
                    None => bail!("CA_CERT does not contain a jurisdiction code in the custom field VX_CUSTOM_CERT_FIELD_JURISDICTION"),
                };

        match JurisdictionCode::try_from(raw_jurisdiction_code.clone()) {
            Ok(jurisdiction_code) => Ok(jurisdiction_code),
            Err(_) => bail!("Invalid jurisdiction code: {raw_jurisdiction_code}"),
        }
    }
}
