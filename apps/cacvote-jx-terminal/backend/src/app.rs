//! Application definition, including all HTTP route handlers.
//!
//! Route handlers are bundled via [`setup`] into an [`axum::Router`], which can then be run
//! using [`run`] at the configured port (see [`config`][`super::config`]).

use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use async_stream::try_stream;
use auth_rs::card_details::CardDetails;
use axum::response::sse::{Event, KeepAlive};
use axum::response::{Response, Sse};
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Json, Router,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use futures_core::Stream;
use serde_json::json;
use sqlx::PgPool;
use tokio::time::sleep;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::Level;
use types_rs::cacvote::jx::AppData;
use types_rs::cacvote::{jx, ServerId};
use types_rs::election::ElectionDefinition;

use crate::config::{Config, MAX_REQUEST_SIZE};
use crate::db::{self, get_app_data};
use crate::smartcard::{self, StatusGetter};

type AppState = (Config, PgPool, smartcard::StatusGetter);

/// Prepares the application with all the routes. Run the application with
/// `app::run(…)` once you have it.
pub(crate) fn setup(
    pool: PgPool,
    config: Config,
    smartcard_status: smartcard::StatusGetter,
) -> Router {
    let _entered = tracing::span!(Level::DEBUG, "Setting up application").entered();

    let router = match &config.public_dir {
        Some(public_dir) => Router::new().fallback_service(
            ServeDir::new(public_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(public_dir.join("index.html"))),
        ),
        None => {
            tracing::info!("No PUBLIC_DIR configured, serving no files");
            Router::new()
        }
    };

    router
        .route("/api/status", get(get_status))
        .route("/api/status-stream", get(get_status_stream))
        .route("/api/jurisdictions", get(get_jurisdictions))
        .route("/api/elections", post(create_election))
        .route("/api/registrations", post(create_registration))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_SIZE))
        .layer(TraceLayer::new_for_http())
        .with_state((config, pool, smartcard_status))
}

/// Runs an application built by `app::setup(…)`.
pub(crate) async fn run(app: Router, config: &Config) -> color_eyre::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
    tracing::info!("Server listening at http://{addr}/");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_status_stream(
    State((_, pool, smartcard_status)): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut last_app_data: Option<AppData> = None;

    fn data_to_yield<'a>(
        last_app_data: &Option<AppData>,
        current_app_data: &'a AppData,
    ) -> Option<&'a AppData> {
        match (last_app_data, current_app_data) {
            (None, _) => Some(current_app_data),
            (Some(last_app_data), current_app_data) if last_app_data != current_app_data => {
                Some(current_app_data)
            }
            _ => None,
        }
    }

    async fn get_current_app_data(
        pool: &PgPool,
        auth_status: Option<CardDetails>,
        smartcard_status: &smartcard::StatusGetter,
    ) -> color_eyre::Result<AppData> {
        match auth_status {
            Some(auth_status) => {
                let mut connection = pool.acquire().await?;
                let app_data =
                    get_app_data(&mut connection, auth_status.jurisdiction_code()).await?;
                Ok(AppData::LoggedIn {
                    auth: auth_status.user(),
                    app_data,
                })
            }
            None => Ok(AppData::LoggedOut {
                auth: smartcard_status.get(),
            }),
        }
    }

    Sse::new(try_stream! {
        loop {
            let auth_status = smartcard_status.get_card_details();
            let current_app_data = match get_current_app_data(&pool, auth_status, &smartcard_status).await {
                Ok(current_app_data) => current_app_data,
                Err(e) => {
                    tracing::error!("error getting current app data: {e}");
                    AppData::LoggedOut { auth: smartcard_status.get() }
                }
            };

            if let Some(new_app_data) = data_to_yield(&last_app_data, &current_app_data) {
                yield Event::default().json_data(new_app_data).unwrap();
                last_app_data = Some(current_app_data);
            }

            sleep(Duration::from_secs(1)).await;
        }
    })
    .keep_alive(KeepAlive::default())
}

async fn get_jurisdictions(State((_, pool, _)): State<AppState>) -> impl IntoResponse {
    let mut connection = pool.acquire().await.map_err(into_internal_error)?;

    let jurisdictions = db::get_jurisdictions(&mut connection)
        .await
        .map_err(into_internal_error)?;

    Ok::<_, Response>(Json(json!({ "jurisdictions": jurisdictions })))
}

async fn authenticate(pool: &PgPool, status_getter: &StatusGetter) -> Option<ServerId> {
    let mut connection = pool.acquire().await.ok()?;
    let card_details = status_getter.get_card_details()?;
    let jurisdiction_code = card_details.jurisdiction_code();
    db::get_jurisdiction_id_for_code(&mut connection, &jurisdiction_code)
        .await
        .ok()?
}

async fn create_election(
    State((config, pool, status_getter)): State<AppState>,
    create_election_data: Json<jx::CreateElectionData>,
) -> impl IntoResponse {
    let election_definition: ElectionDefinition = create_election_data
        .election_data
        .parse()
        .map_err(into_internal_error)?;
    let mut connection = pool.acquire().await.map_err(into_internal_error)?;
    let Some(jurisdiction_id) = authenticate(&pool, &status_getter).await else {
        return Ok::<_, Response>(StatusCode::UNAUTHORIZED);
    };

    db::add_election(
        &mut connection,
        &config,
        jurisdiction_id,
        election_definition,
        &create_election_data.return_address,
    )
    .await
    .map_err(into_internal_error)?;

    Ok::<_, Response>(StatusCode::CREATED)
}

async fn create_registration(
    State((config, pool, _)): State<AppState>,
    registration: Json<jx::CreateRegistrationData>,
) -> impl IntoResponse {
    let ballot_style_id = &registration.ballot_style_id;
    let precinct_id = &registration.precinct_id;

    let mut connection = pool.acquire().await.map_err(into_internal_error)?;

    db::create_registration(
        &mut connection,
        &config,
        registration.registration_request_id,
        registration.election_id,
        precinct_id,
        ballot_style_id,
    )
    .await
    .map_err(into_internal_error)?;

    Ok::<_, Response>(StatusCode::CREATED)
}

fn into_internal_error(e: impl std::fmt::Display) -> Response {
    tracing::error!("internal error: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "error": format!("{e}")
        })),
    )
        .into_response()
}
