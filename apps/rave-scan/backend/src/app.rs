//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

use async_stream::try_stream;
use axum::{
    extract::{DefaultBodyLimit, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    routing::{get, post},
    Json, Router,
};
use central_scanner::scan;
use futures_core::Stream;
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::time::sleep;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;
use types_rs::election::PartialElectionHash;
use types_rs::rave::ClientId;

use crate::config::{MAX_REQUEST_SIZE, PORT, VX_MACHINE_ID};
use crate::db::{self, ScannedBallot, ScannedBallotStats};
use crate::sheets::decode_page_from_image;

/// Prepares the application with all the routes. Run the application with
/// `app::run(…)` once you have it.
pub(crate) async fn setup(pool: PgPool) -> color_eyre::Result<Router> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up application").entered();

    let dist_path = Path::new("../frontend/dist");
    let _ = std::fs::create_dir_all(dist_path);

    Ok(Router::new()
        .fallback_service(
            ServeDir::new(dist_path)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(dist_path.join("index.html"))),
        )
        .route("/api/status", get(get_status))
        .route("/api/status-stream", get(get_status_stream))
        .route("/api/scan", post(do_scan))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state(pool))
}

/// Runs an application built by `app::setup(…)`.
pub(crate) async fn run(app: Router) -> color_eyre::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), *PORT);
    tracing::info!("Server listening at http://{addr}/");
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), *PORT))
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub(crate) struct ScannedCard {
    cvr_data: Vec<u8>,
    election_hash: PartialElectionHash,
}

pub(crate) async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

pub(crate) async fn get_status_stream(
    State(pool): State<PgPool>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut last_scanned_ballot_stats = ScannedBallotStats::default();

    Sse::new(try_stream! {
        loop {
            let mut connection = pool.acquire().await.unwrap();
            if let Ok(scanned_ballot_stats) = db::get_scanned_ballot_stats(&mut connection).await {
                if scanned_ballot_stats != last_scanned_ballot_stats {
                    last_scanned_ballot_stats = scanned_ballot_stats.clone();
                    yield Event::default().json_data(&json!({
                        "status": "ok",
                        "stats": scanned_ballot_stats,
                    })).unwrap();
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .keep_alive(KeepAlive::default())
}

pub(crate) async fn do_scan(State(pool): State<PgPool>) -> Json<Vec<ScannedCard>> {
    let mut connection = pool.acquire().await.unwrap();
    let (tx, rx) = channel();

    let handle = std::thread::spawn(move || {
        let session = scan(PathBuf::from("/tmp")).unwrap();
        for (side_a_path, side_b_path) in session {
            tx.send((side_a_path, side_b_path)).expect("send() failed");
        }
    });

    let elections = db::get_elections(&mut connection, None).await.unwrap();

    let mut cards = vec![];
    for (side_a_path, side_b_path) in rx {
        let (side_a_result, side_b_result) = rayon::join(
            move || decode_page_from_image(image::open(side_a_path).unwrap().to_luma8()),
            move || decode_page_from_image(image::open(side_b_path).unwrap().to_luma8()),
        );

        match (side_a_result, side_b_result) {
            (Err(side_a_err), Err(side_b_err)) => {
                tracing::error!("Both sides failed: {side_a_err:?}, {side_b_err:?}");
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
        if let Some(election) = elections.iter().find(|election| {
            card.election_hash
                .matches_election_hash(&election.election_hash)
        }) {
            let decoded_cvr =
                ballot_encoder_rs::decode(&election.definition.election, card.cvr_data.as_slice())
                    .unwrap();
            let cast_vote_record = serde_json::to_string(&decoded_cvr.cvr).unwrap();

            let scanned_ballot_id = ClientId::new();
            let scanned_ballot = ScannedBallot {
                id: scanned_ballot_id,
                server_id: None,
                client_id: scanned_ballot_id,
                machine_id: VX_MACHINE_ID.to_string(),
                election_id: election.id,
                cast_vote_record: cast_vote_record.into_bytes(),
                created_at: OffsetDateTime::now_utc(),
            };

            match db::add_scanned_ballot(&mut connection, scanned_ballot).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Failed to insert scanned ballot: {e}");
                }
            }
        } else {
            tracing::error!(
                "No election found for card with hash {}",
                card.election_hash
            );
        }
    }

    handle.join().unwrap();

    Json(cards)
}
