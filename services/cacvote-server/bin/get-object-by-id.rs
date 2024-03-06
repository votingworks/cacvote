use clap::Parser;
use serde_json::Value;
use types_rs::cacvote::Payload;
use url::Url;
use uuid::Uuid;

use cacvote_server::client::Client;

#[derive(Debug, Parser)]
struct Opts {
    object_id: Uuid,

    #[clap(
        long,
        env = "CACVOTE_SERVER_URL",
        default_value = "http://localhost:8000"
    )]
    cacvote_server_url: Url,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let opts = Opts::parse();
    let client = Client::new(opts.cacvote_server_url.clone());

    let signed_object = match client.get_object_by_id(opts.object_id).await? {
        Some(signed_object) => signed_object,
        None => {
            println!("no object found with ID {}", opts.object_id);
            return Ok(());
        }
    };

    println!("signed object: {signed_object:#?}");

    let payload: Payload = serde_json::from_slice(&signed_object.payload)?;
    println!("object payload: {payload:#?}");

    let data: Value = serde_json::from_slice(&payload.data)?;
    println!("payload data: {data:#?}");

    Ok(())
}
