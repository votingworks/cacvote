use std::fs;

use cacvote_server::{config::Config, db, log};
use clap::Parser;

#[derive(Debug, Parser)]
struct EnrollmentOptions {
    /// The Voter Terminal machine identifier.
    machine_identifier: String,

    /// The path to the public key of the Voter Terminal machine (in PEM format).
    public_key_path: String,

    /// URL of the PostgreSQL database, e.g. `postgres://user:pass@host:port/dbname`.
    #[arg(long, env = "DATABASE_URL")]
    pub(crate) database_url: String,

    /// Log level.
    #[arg(long, env = "LOG_LEVEL", default_value = "info")]
    pub(crate) log_level: tracing::Level,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let _ = dotenvy::from_filename(".env.local");
    dotenvy::dotenv()?;

    let options = EnrollmentOptions::parse();
    let config = Config {
        database_url: options.database_url,
        port: 0, // unused in this context
        log_level: options.log_level,
        ca_cert: "unused".to_string(), // unused in this context
    };
    log::setup(&config)?;
    let pool = db::setup(&config).await?;
    let mut conn = pool.acquire().await?;

    let certificate = fs::read(options.public_key_path)?;

    // verify that the data is a valid stack of X509 certificates
    openssl::x509::X509::from_pem(&certificate)?;

    let machine = db::create_machine(&mut conn, &options.machine_identifier, &certificate).await?;
    println!("✅ Machine enrolled! ID={id}", id = machine.id);

    Ok(())
}
