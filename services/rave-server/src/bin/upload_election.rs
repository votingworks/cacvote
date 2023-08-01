use std::env::args;
use types_rs::rave::{client::input::Election, ClientId, RaveMarkSyncInput, RaveMarkSyncOutput};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let rave_url = reqwest::Url::parse(std::env::var("RAVE_URL")?.as_str())?;
    let mut sync_input = RaveMarkSyncInput::default();
    let sync_endpoint = rave_url.join("/api/sync")?;

    let election_path = args().skip(1).next().expect("election path is required");
    let election_data = std::fs::read_to_string(election_path)?;
    let election = Election {
        client_id: ClientId::new(),
        machine_id: "manual".to_string(),
        election: election_data.parse()?,
    };
    sync_input.elections.push(election);

    let sync_output = sync(sync_endpoint, &sync_input).await?;

    println!("{:#?}", sync_output);

    Ok(())
}

pub(crate) async fn sync(
    endpoint: reqwest::Url,
    sync_input: &RaveMarkSyncInput,
) -> color_eyre::eyre::Result<RaveMarkSyncOutput> {
    let client = reqwest::Client::new();
    Ok(client
        .post(endpoint)
        .json(sync_input)
        .send()
        .await?
        .json::<RaveMarkSyncOutput>()
        .await?)
}
