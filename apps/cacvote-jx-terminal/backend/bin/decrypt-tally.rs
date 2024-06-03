use std::{iter::once, path::PathBuf, time::Duration};

use clap::Parser;
use color_eyre::eyre::bail;
use electionguard_rs::{config, tally};
use sqlx::postgres::PgPoolOptions;
use types_rs::cacvote::Payload;
use url::Url;
use uuid::Uuid;

use cacvote_server_client::Client;

#[derive(Debug, Parser)]
struct Opts {
    object_id: Uuid,

    #[clap(
        long,
        env = "CACVOTE_SERVER_URL",
        default_value = "http://localhost:8000"
    )]
    cacvote_server_url: Url,

    #[clap(long, env = "DATABASE_URL")]
    database_url: String,

    #[clap(long, env = "EG_CLASSPATH")]
    electionguard_classpath: PathBuf,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let opts = Opts::parse();
    let client = Client::new(opts.cacvote_server_url.clone());

    let cast_ballot_object = match client.get_object_by_id(opts.object_id).await? {
        Some(object) => object,
        None => {
            bail!("no CastBallot found with ID {}", opts.object_id);
        }
    };

    let Payload::CastBallot(cast_ballot) = cast_ballot_object.try_to_inner()? else {
        bail!("object with ID {} is not a CastBallot", opts.object_id);
    };

    let election_object = match client
        .get_object_by_id(cast_ballot.election_object_id)
        .await?
    {
        Some(object) => object,
        None => {
            bail!(
                "no Election found with ID {}",
                cast_ballot.election_object_id
            );
        }
    };

    let Payload::Election(election) = election_object.try_to_inner()? else {
        bail!(
            "object with ID {} is not an Election",
            cast_ballot.election_object_id
        );
    };

    let encrypted_tally = tally::accumulate(
        &opts.electionguard_classpath,
        &election.electionguard_election_metadata_blob,
        once(cast_ballot.electionguard_encrypted_ballot.as_slice()),
    )?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&opts.database_url)
        .await?;
    let mut connection = pool.acquire().await?;

    let record = sqlx::query!(
        r#"
        SELECT private_key
        FROM eg_private_keys
        WHERE election_object_id = $1
        "#,
        cast_ballot.election_object_id
    )
    .fetch_one(&mut *connection)
    .await?;

    let election_config = config::ElectionConfig {
        public_metadata_blob: election.electionguard_election_metadata_blob,
        private_metadata_blob: record.private_key,
    };

    let decrypted_tally_bytes = tally::decrypt(
        &opts.electionguard_classpath,
        &election_config,
        &encrypted_tally,
    )?;
    let decrypted_tally_json = std::str::from_utf8(decrypted_tally_bytes.as_slice())?;

    println!("{decrypted_tally_json}");

    Ok(())
}
