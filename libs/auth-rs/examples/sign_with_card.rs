use std::{io::Write, path::PathBuf};

use clap::Parser;
use tracing::Level;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

use auth_rs::{vx_card::CARD_VX_ADMIN_CERT, CardReader, Event, Watcher};

#[derive(clap::Parser)]
struct SignWithCardArgs {
    #[clap(long)]
    no_pin: bool,

    path: PathBuf,
}

fn main() -> color_eyre::Result<()> {
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

    let ctx = pcsc::Context::establish(pcsc::Scope::User).unwrap();
    let mut watcher = Watcher::watch();
    let mut card_reader: Option<CardReader> = None;

    println!("Insert a card to sign…");

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
        let pin = if args.no_pin {
            None
        } else {
            print!("Enter the PIN to sign the data: ");
            std::io::stdout().flush()?;
            let mut pin = String::new();
            std::io::stdin().read_line(&mut pin).unwrap();
            let pin = pin.trim();
            Some(pin.to_owned())
        };

        match card.sign(
            CARD_VX_ADMIN_CERT,
            &std::fs::read(args.path)?,
            pin.as_deref(),
        ) {
            Ok((signature, _)) => {
                println!("{signature:02x?}");
            }
            Err(error) => {
                eprintln!("Error: {error}");
            }
        }
    }

    Ok(())
}
