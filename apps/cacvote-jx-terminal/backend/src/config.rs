//! Application configuration.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use auth_rs::{card_details::extract_field_value, certs::VX_CUSTOM_CERT_FIELD_JURISDICTION};
use cacvote_server_client::{signer, AnySigner};
use clap::Parser;
use color_eyre::eyre::{bail, Context};
use openssl::{hash::MessageDigest, sign::Verifier, x509::X509};
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

    /// Certificate authority certificate files, used to validate the client
    /// certificates from a CAC.
    #[arg(long, env = "CAC_ROOT_CA_CERTS", value_delimiter = ',')]
    pub(crate) cac_root_ca_certs: Vec<PathBuf>,

    /// Certificate associated with this machine's unique private key. Issued by
    /// the VX CA.
    #[arg(long, env = "MACHINE_CERT")]
    pub(crate) machine_cert: PathBuf,

    /// Signer to use for signing server request payloads.
    #[arg(long, env = "SIGNER")]
    pub(crate) signer: signer::Description,

    /// VX CA certificate.
    #[arg(long, env = "VX_CA_CERT")]
    pub(crate) vx_cert_authority_cert: PathBuf,

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
    pub(crate) fn verify(&self) -> color_eyre::Result<()> {
        // Verify that the MACHINE_CERT is signed by the VX CA.
        let machine_cert = self.machine_cert()?;
        let vx_cert_authority_cert = self.vx_cert_authority_cert()?;
        let vx_cert_authority_public_key = vx_cert_authority_cert.public_key()?;
        machine_cert.verify(&vx_cert_authority_public_key)?;

        // Verify that the MACHINE_CERT contains a jurisdiction code.
        let _ = self.jurisdiction_code()?;

        // Verify that the signer is valid.
        let message = b"test";
        let signer = self.signer()?;
        let signature = signer.sign(message)?;
        let machine_public_key = machine_cert.public_key()?;
        let mut verifier = Verifier::new(MessageDigest::sha256(), &machine_public_key)?;
        verifier.update(message)?;
        if !verifier.verify(&signature)? {
            bail!("signature from SIGNER is not verifiable by MACHINE_CERT");
        }

        Ok(())
    }

    pub(crate) fn cac_root_ca_store(&self) -> color_eyre::Result<openssl::x509::store::X509Store> {
        let mut builder = openssl::x509::store::X509StoreBuilder::new()?;

        for ca_cert in &self.cac_root_ca_certs {
            builder.add_cert(load_cert(ca_cert)?)?;
        }

        Ok(builder.build())
    }

    pub(crate) fn machine_cert(&self) -> color_eyre::Result<openssl::x509::X509> {
        load_cert(&self.machine_cert)
    }

    pub(crate) fn vx_cert_authority_cert(&self) -> color_eyre::Result<openssl::x509::X509> {
        load_cert(&self.vx_cert_authority_cert)
    }

    /// Returns the jurisdiction code from the MACHINE_CERT certificate.
    pub(crate) fn jurisdiction_code(&self) -> color_eyre::Result<JurisdictionCode> {
        let raw_jurisdiction_code = match extract_field_value(&self.machine_cert()?, VX_CUSTOM_CERT_FIELD_JURISDICTION)
                .context("Unable to extract jurisdiction code from MACHINE_CERT")? {
                    Some(value) => value,
                    None => bail!("MACHINE_CERT does not contain a jurisdiction code in the custom field VX_CUSTOM_CERT_FIELD_JURISDICTION"),
                };

        match JurisdictionCode::try_from(raw_jurisdiction_code.clone()) {
            Ok(jurisdiction_code) => Ok(jurisdiction_code),
            Err(_) => bail!("Invalid jurisdiction code: {raw_jurisdiction_code}"),
        }
    }

    pub(crate) fn signer(&self) -> color_eyre::Result<AnySigner> {
        AnySigner::try_from(&self.signer)
    }

    pub(crate) fn sign(&self, payload: &[u8]) -> color_eyre::Result<(Vec<u8>, X509)> {
        let signer = self.signer()?;
        let signature = signer.sign(payload)?;
        let cert = self.machine_cert()?;
        Ok((signature, cert))
    }
}

fn load_cert<P>(path: P) -> color_eyre::Result<openssl::x509::X509>
where
    P: AsRef<Path>,
{
    let ca_cert = std::fs::read(path)?;
    Ok(openssl::x509::X509::from_pem(&ca_cert)
        .or_else(|_| openssl::x509::X509::from_der(&ca_cert))?)
}
