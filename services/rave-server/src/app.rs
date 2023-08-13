//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing::Level;
use types_rs::rave::{client, RaveServerSyncInput, RaveServerSyncOutput};

use crate::{
    config::{MAX_REQUEST_SIZE, PORT},
    db,
};

/// Prepares the application to be run within an HTTP server.
///
/// Requires a [`PgPool`] from [`db::setup`]. Run the application with [`run`]
/// with the result of this function.
pub(crate) async fn setup(pool: PgPool) -> color_eyre::Result<Router> {
    let _entered = tracing::span!(Level::DEBUG, "Setting up application").entered();
    Ok(Router::new()
        .route("/api/status", get(get_status))
        .route("/api/admins", post(create_admin))
        .route("/api/sync", post(do_sync))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state(pool))
}

/// Create and run an HTTP server using the provided application at the port
/// from [`config`][`super::config`].
pub(crate) async fn run(app: Router) -> color_eyre::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), *PORT);
    tracing::info!("Server listening at http://{addr}/");
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), *PORT))
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

/// Always responds with a successful status. Used to check whether the server
/// is running.
pub(crate) async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

/// Synchronizes data between a client and the server.
pub(crate) async fn do_sync(
    State(pool): State<PgPool>,
    Json(input): Json<RaveServerSyncInput>,
) -> Result<Json<RaveServerSyncOutput>, impl IntoResponse> {
    let mut txn = match pool.begin().await {
        Ok(txn) => txn,
        Err(e) => {
            tracing::error!("Failed to begin transaction: {}", e);
            return Err(Json(json!({
                "success": false,
                "error": format!("failed to begin transaction: {}", e)
            })));
        }
    };

    let RaveServerSyncInput {
        last_synced_registration_request_id,
        last_synced_registration_id,
        last_synced_election_id,
        last_synced_printed_ballot_id,
        last_synced_scanned_ballot_id,
        registration_requests,
        elections,
        registrations,
        printed_ballots,
        scanned_ballots,
    } = input;

    for client_request in registration_requests.into_iter() {
        let server_request: client::input::RegistrationRequest = client_request;
        let result = db::add_registration_request_from_client(&mut txn, &server_request).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert registration request: {}", e);
        }
    }

    for election in elections.into_iter() {
        let result = db::add_election(&mut txn, election).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert election: {}", e);
        }
    }

    for registration in registrations.into_iter() {
        let result = db::add_registration_from_client(&mut txn, registration).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert registration: {}", e);
        }
    }

    for printed_ballot in printed_ballots.into_iter() {
        let result = db::add_printed_ballot_from_client(&mut txn, printed_ballot).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert printed ballot: {}", e);
        }
    }

    for scanned_ballot in scanned_ballots.into_iter() {
        let result = db::add_scanned_ballot_from_client(&mut txn, scanned_ballot).await;

        if let Err(e) = result {
            tracing::error!("Failed to insert scanned ballot: {}", e);
        }
    }

    let get_admins_result = db::get_admins(&mut txn).await;
    let admins = match get_admins_result {
        Err(e) => {
            tracing::error!("Failed to get admins: {}", e);
            return Err(Json(json!({ "error": e.to_string() })));
        }
        Ok(admins) => admins,
    };

    let get_elections_result = db::get_elections(&mut txn, last_synced_election_id).await;
    let elections = match get_elections_result {
        Err(e) => {
            tracing::error!("Failed to get elections: {}", e);
            return Err(Json(json!({ "error": e.to_string() })));
        }
        Ok(elections) => elections,
    };

    let get_registration_requests_result =
        db::get_registration_requests(&mut txn, last_synced_registration_request_id).await;
    let registration_requests = match get_registration_requests_result {
        Err(e) => {
            tracing::error!("Failed to get registration requests: {}", e);
            return Err(Json(json!({ "error": e.to_string() })));
        }
        Ok(registration_requests) => registration_requests,
    };

    let get_registrations_result =
        db::get_registrations(&mut txn, last_synced_registration_id).await;
    let registrations = match get_registrations_result {
        Err(e) => {
            tracing::error!("Failed to get registrations: {}", e);
            return Err(Json(json!({ "error": e.to_string() })));
        }
        Ok(registrations) => registrations,
    };

    let printed_ballots =
        match db::get_printed_ballots(&mut txn, last_synced_printed_ballot_id).await {
            Err(e) => {
                tracing::error!("Failed to get printed ballots: {}", e);
                return Err(Json(json!({ "error": e.to_string() })));
            }
            Ok(ballots) => ballots,
        };

    let scanned_ballots =
        match db::get_scanned_ballots(&mut txn, last_synced_scanned_ballot_id).await {
            Err(e) => {
                tracing::error!("Failed to get scanned ballots: {}", e);
                return Err(Json(json!({ "error": e.to_string() })));
            }
            Ok(ballots) => ballots,
        };

    let output = RaveServerSyncOutput {
        admins: admins.into_iter().map(|admin| admin.into()).collect(),
        elections: elections
            .into_iter()
            .map(|election| election.into())
            .collect(),
        registration_requests: registration_requests
            .into_iter()
            .map(|registration_request| registration_request.into())
            .collect(),
        registrations: registrations
            .into_iter()
            .map(|registration| registration.into())
            .collect(),
        printed_ballots: printed_ballots
            .into_iter()
            .map(|ballot| ballot.into())
            .collect(),
        scanned_ballots: scanned_ballots
            .into_iter()
            .map(|ballot| ballot.into())
            .collect(),
    };

    if let Err(err) = txn.commit().await {
        tracing::error!("Failed to commit transaction: {}", err);
        return Err(Json(json!({ "error": err.to_string() })));
    }

    Ok(Json(output))
}

/// Creates an admin user. Admins have elevated privileges when logged into
/// client machines.
pub(crate) async fn create_admin(
    State(pool): State<PgPool>,
    Json(input): Json<client::input::Admin>,
) -> impl IntoResponse {
    let input = input;
    let mut connection = match pool.acquire().await {
        Ok(connection) => connection,
        Err(err) => {
            tracing::error!("Failed to acquire connection: {}", err);
            return Err(Json(json!({ "error": err.to_string() })));
        }
    };
    let result = db::add_admin(&mut connection, input).await;

    result.map_or_else(
        |err| {
            tracing::error!("Failed to create admin: {}", err);
            Err(Json(json!({ "error": err.to_string() })))
        },
        |_| Ok(StatusCode::CREATED),
    )
}
