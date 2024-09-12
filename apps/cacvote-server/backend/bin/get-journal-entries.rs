use std::path::PathBuf;

use auth_rs::{card_details::extract_field_value, certs::VX_CUSTOM_CERT_FIELD_JURISDICTION};
use cacvote_server::client::Client;
use cacvote_server_client::{PrivateKeySigner, TpmSigner};
use clap::Parser;
use color_eyre::eyre::eyre;
use openssl::{pkey::PKey, x509::X509};
use reqwest::Url;
use types_rs::cacvote::JurisdictionCode;
use uuid::Uuid;

#[derive(Parser)]
struct Opts {
    #[clap(long)]
    since: Option<Uuid>,

    #[clap(
        long,
        env = "CACVOTE_SERVER_URL",
        default_value = "http://localhost:8000"
    )]
    cacvote_server_url: Url,

    #[clap(long, env = "SIGNING_CERT")]
    signing_cert: PathBuf,

    #[clap(long, env = "SIGNER")]
    signer: SignerDescription,
}

impl Opts {
    fn signing_cert(&self) -> color_eyre::Result<X509> {
        let pem = std::fs::read(&self.signing_cert)?;
        let signing_cert = X509::from_pem(&pem)?;
        Ok(signing_cert)
    }

    fn jurisdiction_code(&self) -> color_eyre::Result<JurisdictionCode> {
        let signing_cert = self.signing_cert()?;
        let jurisdiction_code =
            extract_field_value(&signing_cert, VX_CUSTOM_CERT_FIELD_JURISDICTION)?
                .ok_or_else(|| eyre!("signing certificate does not have a jurisdiction code"))?;
        JurisdictionCode::try_from(jurisdiction_code)
            .map_err(|e| eyre!("invalid jurisdiction code: {e}"))
    }
}

#[derive(Debug, Clone)]
enum SignerDescription {
    File(PathBuf),
    Tpm(i32),
}

impl std::str::FromStr for SignerDescription {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("tpm:") {
            Some(handle) => match handle.strip_prefix("0x") {
                Some(handle) => {
                    let tpm = i32::from_str_radix(handle, 16).map_err(|e| e.to_string())?;
                    Ok(Self::Tpm(tpm))
                }
                None => {
                    let tpm = handle.parse::<i32>().map_err(|e| e.to_string())?;
                    Ok(Self::Tpm(tpm))
                }
            },
            None => Ok(Self::File(PathBuf::from(s))),
        }
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    dotenvy::dotenv()?;

    let opts = Opts::parse();
    let signing_cert = opts.signing_cert()?;
    let mut client = Client::new(
        opts.cacvote_server_url.clone(),
        signing_cert.clone(),
        match opts.signer {
            SignerDescription::File(ref path) => {
                let pem = std::fs::read(path)?;
                Box::new(PrivateKeySigner::new(PKey::private_key_from_pem(&pem)?))
            }
            SignerDescription::Tpm(handle) => Box::new(TpmSigner::new(handle.try_into()?)),
        },
    );

    let entries = client
        .get_journal_entries(opts.since.as_ref(), Some(&opts.jurisdiction_code()?))
        .await?;

    println!("entries: {entries:#?}");

    Ok(())
}
