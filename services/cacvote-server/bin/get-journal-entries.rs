use cacvote_server::client::Client;
use clap::Parser;
use reqwest::Url;
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
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv()?;

    let opts = Opts::parse();
    let client = Client::new(opts.cacvote_server_url.clone());
    let entries = client.get_journal_entries(opts.since).await?;

    println!("entries: {entries:#?}");

    Ok(())
}
