use std::{
    io::Write,
    path::{Path, PathBuf},
    process,
};

use clap::Parser;
use tracing::Level;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

use auth_rs::{
    async_card::AsyncCard, vx_card::VxCard, vx_card::CARD_VX_ADMIN_CERT, Event, Watcher,
};

#[derive(clap::Parser)]
struct SignWithCardArgs {
    #[clap(long)]
    no_pin: bool,

    path: PathBuf,

    /// VX CA certificate.
    #[arg(long, env = "VX_CA_CERT")]
    pub(crate) vx_cert_authority_cert: PathBuf,

    /// VxAdmin CA certificate.
    #[arg(long, env = "VX_ADMIN_CA_CERT")]
    pub(crate) vx_admin_cert_authority_cert: PathBuf,
}

impl SignWithCardArgs {
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

    let args = SignWithCardArgs::parse();

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

    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let mut watcher = Watcher::watch();
    let mut card: Option<VxCard> = None;

    println!("Insert a card to sign…");

    while let Some(event) = watcher.recv().await {
        match event? {
            Event::CardInserted { reader_name } => {
                card = Some(VxCard::new(
                    args.vx_cert_authority_cert()?,
                    args.vx_admin_cert_authority_cert()?,
                    AsyncCard::connect(&ctx, &reader_name)?,
                ));
                break;
            }
            _ => {}
        }
    }

    if let Some(card) = card {
        watcher.stop().await;
        let pin = if args.no_pin {
            None
        } else {
            print!("Enter the PIN to sign the data: ");
            std::io::stdout().flush()?;
            let mut pin = String::new();
            std::io::stdin().read_line(&mut pin)?;
            let pin = pin.trim();
            Some(pin.to_owned())
        };

        match card
            .sign(
                CARD_VX_ADMIN_CERT,
                &std::fs::read(args.path)?,
                pin.as_deref(),
            )
            .await
        {
            Ok((signature, _)) => {
                println!("{signature:02x?}");
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
