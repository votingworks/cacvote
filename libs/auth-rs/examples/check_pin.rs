use std::io::Write;

use tracing::Level;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

use auth_rs::{CardReader, Event, Watcher};

fn main() -> color_eyre::Result<()> {
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

    let ctx = pcsc::Context::establish(pcsc::Scope::User).unwrap();
    let mut watcher = Watcher::watch();
    let mut card_reader: Option<CardReader> = None;

    println!("Insert a card to check its PIN…");

    for event in watcher.events() {
        match event {
            Ok(Event::CardInserted { reader_name }) => {
                card_reader = Some(CardReader::new(ctx.clone(), reader_name));
                break;
            }
            Err(error) => {
                eprintln!("Error: {}", error);
                break;
            }
            _ => {}
        }
    }

    if let Some(card_reader) = card_reader {
        watcher.stop();
        let card = card_reader.get_card()?;
        print!("Enter the PIN to check its validity: ");
        std::io::stdout().flush()?;
        let mut pin = String::new();
        std::io::stdin().read_line(&mut pin).unwrap();
        let pin = pin.trim();

        match card.check_pin(pin) {
            Ok(()) => {
                println!("OK: PIN is valid");
            }
            Err(error) => {
                eprintln!("Error: {error}");
            }
        }
    }

    Ok(())
}
