use types_rs::rave::{RaveMarkSyncInput, RaveMarkSyncOutput};

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
