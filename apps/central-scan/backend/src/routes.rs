use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

use rocket::fs::{relative, NamedFile};

use central_scanner::scan;
use rocket::http::{ContentType, Status};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::{json, Json};
use rocket::serde::Serialize;
use rocket::tokio::time::sleep;
use rocket_db_pools::Connection;
use time::OffsetDateTime;
use types_rs::rave::{ClientId, RaveMarkSyncInput, RaveMarkSyncOutput};

use crate::cards::decode_page_from_image;
use crate::db::{self, ScannedBallot, ScannedBallotStats};
use crate::sync::sync;

lazy_static::lazy_static! {
    static ref RAVE_URL: reqwest::Url = reqwest::Url::parse(
        std::env::var("RAVE_URL")
            .expect("RAVE_URL must be set")
            .as_str(),
    )
    .expect("RAVE_URL must be a valid URL");
}

lazy_static::lazy_static! {
    static ref VX_MACHINE_ID: String = std::env::var("VX_MACHINE_ID")
        .expect("VX_MACHINE_ID must be set");
}

#[derive(Debug, Serialize)]
pub struct ScannedCard {
    cvr_data: Vec<u8>,
    election_hash: String,
}

#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(relative!("static/index.html")).await.ok()
}

#[get("/api/status")]
pub(crate) async fn get_status(mut db: Connection<db::Db>) -> EventStream![Event] {
    let mut last_scanned_ballot_stats = ScannedBallotStats::default();

    EventStream! {
        loop {
            if let Ok(scanned_ballot_stats) = db::get_scanned_ballot_stats(&mut db).await {
                if scanned_ballot_stats != last_scanned_ballot_stats {
                    last_scanned_ballot_stats = scanned_ballot_stats.clone();
                    yield Event::json(&json!({
                        "status": "ok",
                        "stats": scanned_ballot_stats,
                    }));
                }
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

#[post("/api/scan")]
pub(crate) async fn do_scan(mut db: Connection<db::Db>) -> Json<Vec<ScannedCard>> {
    let (tx, rx) = channel();

    let handle = std::thread::spawn(move || {
        let session = scan(PathBuf::from("/tmp")).unwrap();
        for (side_a_path, side_b_path) in session {
            tx.send((side_a_path, side_b_path)).expect("send() failed");
        }
    });

    let elections = db::get_elections(&mut db, None).await.unwrap();

    let mut cards = vec![];
    for (side_a_path, side_b_path) in rx {
        let (side_a_result, side_b_result) = rayon::join(
            move || {
                let start = std::time::Instant::now();
                let side_a_image = image::open(side_a_path).unwrap().to_luma8();
                eprintln!("A opened in {:?}", start.elapsed());
                let decoded = decode_page_from_image(side_a_image);
                eprintln!("A decoded in {:?}", start.elapsed());
                decoded
            },
            move || {
                let start = std::time::Instant::now();
                let side_b_image = image::open(side_b_path).unwrap().to_luma8();
                eprintln!("side_b opened in {:?}", start.elapsed());
                let decoded = decode_page_from_image(side_b_image);
                eprintln!("side_b decoded in {:?}", start.elapsed());
                decoded
            },
        );

        match (side_a_result, side_b_result) {
            (Err(side_a_err), Err(side_b_err)) => {
                eprintln!("Both sides failed: {:?}, {:?}", side_a_err, side_b_err);
                break;
            }
            (Ok(cvr_data), _) | (_, Ok(cvr_data)) => {
                let (_, election_hash) = ballot_encoder_rs::decode_header(&cvr_data).unwrap();
                cards.push(ScannedCard {
                    election_hash,
                    cvr_data,
                })
            }
        }
    }

    for card in cards.iter() {
        if let Some(election) = elections
            .iter()
            .find(|election| election.election_hash.starts_with(&card.election_hash))
        {
            let decoded_cvr =
                ballot_encoder_rs::decode(&election.definition.election, card.cvr_data.as_slice())
                    .unwrap();

            let scanned_ballot_id = ClientId::new();
            let scanned_ballot = ScannedBallot {
                id: scanned_ballot_id.clone(),
                server_id: None,
                client_id: scanned_ballot_id,
                machine_id: VX_MACHINE_ID.to_string(),
                election_id: election.id.clone(),
                cast_vote_record: decoded_cvr.cvr,
                created_at: OffsetDateTime::now_utc(),
            };

            match db::add_scanned_ballot(&mut db, scanned_ballot).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to insert scanned ballot: {}", e);
                }
            }
        } else {
            error!(
                "No election found for card with hash {}",
                card.election_hash
            );
        }
    }

    handle.join().unwrap();

    Json(cards)
}

#[post("/api/sync")]
pub(crate) async fn do_sync(mut db: Connection<db::Db>) -> (Status, (ContentType, String)) {
    let sync_input = RaveMarkSyncInput {
        last_synced_election_id: match db::get_last_synced_election_id(&mut db).await {
            Ok(last_synced_election_id) => last_synced_election_id,
            Err(e) => {
                return (
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        json!({
                            "error": format!("failed to get last synced election ID: {}", e)
                        })
                        .to_string(),
                    ),
                )
            }
        },
        last_synced_scanned_ballot_id: match db::get_last_synced_scanned_ballot_id(&mut db).await {
            Ok(last_synced_scanned_ballot_id) => last_synced_scanned_ballot_id,
            Err(e) => {
                return (
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        json!({
                            "error": format!("failed to get last synced scanned ballot ID: {}", e)
                        })
                        .to_string(),
                    ),
                )
            }
        },
        scanned_ballots: match db::get_scanned_ballots_to_sync_to_rave_server(&mut db).await {
            Ok(scanned_ballots) => scanned_ballots,
            Err(e) => {
                return (
                    Status::InternalServerError,
                    (
                        ContentType::JSON,
                        json!({
                            "error": format!("failed to get scanned ballots to sync: {}", e)
                        })
                        .to_string(),
                    ),
                )
            }
        },
        ..Default::default()
    };

    let sync_endpoint = RAVE_URL
        .join("/api/sync")
        .expect("failed to construct sync URL");
    let sync_output = sync(sync_endpoint, &sync_input).await.unwrap();

    let RaveMarkSyncOutput {
        admins,
        elections,
        registration_requests,
        registrations,
        printed_ballots,
        scanned_ballots,
    } = sync_output.clone();

    for admin in admins.into_iter() {
        let result = db::add_admin_from_rave_server(&mut db, admin).await;

        if let Err(e) = result {
            error!("Failed to insert admin: {}", e);
        }
    }

    for election in elections.into_iter() {
        let result = db::add_election_from_rave_server(&mut db, election).await;

        if let Err(e) = result {
            error!("Failed to insert election: {}", e);
        }
    }

    info!(
        "ignoring {} registration requests",
        registration_requests.len()
    );
    info!("ignoring {} registrations", registrations.len());
    info!("ignoring {} printed ballots", printed_ballots.len());

    for scanned_ballot in scanned_ballots.into_iter() {
        let result =
            db::add_or_update_scanned_ballot_from_rave_server(&mut db, scanned_ballot).await;

        if let Err(e) = result {
            error!("Failed to insert or update scanned ballot: {}", e);
        }
    }

    (
        Status::Ok,
        (
            ContentType::JSON,
            serde_json::to_string(&sync_output).unwrap(),
        ),
    )
}
