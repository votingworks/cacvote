use std::{
    io::Write,
    path::{Path, PathBuf},
    process,
};

use clap::Parser;
use tracing::Level;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

use auth_rs::{async_card::AsyncCard, vx_card::VxCard, Event, Watcher};

#[derive(Parser)]
struct Config {
    /// VX CA certificate.
    #[arg(long, env = "VX_CA_CERT")]
    pub(crate) vx_cert_authority_cert: PathBuf,

    /// VxAdmin CA certificate.
    #[arg(long, env = "VX_ADMIN_CA_CERT")]
    pub(crate) vx_admin_cert_authority_cert: PathBuf,
}

impl Config {
    pub(crate) fn vx_cert_authority_cert(&self) -> color_eyre::Result<openssl::x509::X509> {
        load_cert(&self.vx_cert_authority_cert)
    }

    pub(crate) fn vx_admin_cert_authority_cert(&self) -> color_eyre::Result<openssl::x509::X509> {
        load_cert(&self.vx_admin_cert_authority_cert)
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

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let stdout_log = tracing_subscriber::fmt::layer().pretty();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    format!(
                        "{}={}",
                        env!("CARGO_PKG_NAME").replace('-', "_"),
                        Level::INFO
                    )
                    .parse()?,
                )
                .from_env_lossy(),
        )
        .with(stdout_log)
        .init();

    let config = Config::parse();
    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let mut watcher = Watcher::watch();
    let mut card: Option<VxCard> = None;

    println!("Insert a card to check its PIN…");

    while let Some(event) = watcher.recv().await {
        match event {
            Ok(Event::CardInserted { reader_name }) => {
                card = Some(VxCard::new(
                    config.vx_cert_authority_cert()?,
                    config.vx_admin_cert_authority_cert()?,
                    AsyncCard::connect(&ctx, &reader_name)?,
                ));
                break;
            }
            Err(error) => {
                eprintln!("Error: {}", error);
                break;
            }
            _ => {}
        }
    }

    if let Some(card) = card {
        watcher.stop().await;
        print!("Enter the PIN to check its validity: ");
        std::io::stdout().flush()?;
        let mut pin = String::new();
        std::io::stdin().read_line(&mut pin)?;
        let pin = pin.trim();

        match card.check_pin(pin).await {
            Ok(()) => {
                println!("OK: PIN is valid");
            }
            Err(error) => {
                eprintln!("Error: {error}");
            }
        }
    }

    // FIXME: why does tokio not exit on its own?
    // Try something from https://tokio.rs/tokio/topics/shutdown#waiting-for-things-to-finish-shutting-down
    process::exit(0);
}
