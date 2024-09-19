use std::{io::Write, process};

use tracing::Level;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

use auth_rs::{async_card::AsyncCard, vx_card::VxCard, Event, Watcher};

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

    let ctx = pcsc::Context::establish(pcsc::Scope::User)?;
    let mut watcher = Watcher::watch();
    let mut card: Option<VxCard> = None;

    println!("Insert a card to check its PIN…");

    while let Some(event) = watcher.recv().await {
        match event {
            Ok(Event::CardInserted { reader_name }) => {
                card = Some(VxCard::new(AsyncCard::connect(&ctx, &reader_name)?));
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
