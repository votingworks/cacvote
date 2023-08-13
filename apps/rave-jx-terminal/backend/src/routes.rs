use std::convert::Infallible;
use std::time::Duration;

use async_stream::try_stream;
use axum::response::sse::{Event, KeepAlive};
use axum::response::{Response, Sse};
use axum::Json;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use futures_core::Stream;
use serde_json::json;
use sqlx::PgPool;
use tokio::time::sleep;
use types_rs::election::ElectionDefinition;
use types_rs::rave::jx;

use crate::db::{self, get_app_data};

pub(crate) async fn get_status() -> impl IntoResponse {
    StatusCode::OK
}

pub(crate) async fn get_status_stream(
    State(pool): State<PgPool>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut last_app_data = jx::AppData::default();

    Sse::new(try_stream! {
        loop {
            let mut connection = pool.acquire().await.unwrap();
            let app_data = get_app_data(&mut connection).await.unwrap();

            if app_data != last_app_data {
                yield Event::default().json_data(&app_data).unwrap();
                last_app_data = app_data;
            }

            sleep(Duration::from_secs(1)).await;
        }
    })
    .keep_alive(KeepAlive::default())
}

pub(crate) async fn create_election(
    State(pool): State<PgPool>,
    election: String,
) -> impl IntoResponse {
    let election_definition: ElectionDefinition =
        election.parse().map_err(|err| into_internal_error(err))?;
    let mut connection = pool
        .acquire()
        .await
        .map_err(|err| into_internal_error(err))?;

    db::add_election(&mut connection, election_definition)
        .await
        .map_err(|err| into_internal_error(err))?;

    Ok::<_, Response>(StatusCode::CREATED)
}

pub(crate) async fn create_registration(
    State(pool): State<PgPool>,
    registration: Json<jx::CreateRegistrationData>,
) -> impl IntoResponse {
    let ballot_style_id = &registration.ballot_style_id;
    let precinct_id = &registration.precinct_id;

    let mut connection = pool
        .acquire()
        .await
        .map_err(|err| into_internal_error(err))?;

    db::create_registration(
        &mut connection,
        registration.registration_request_id,
        registration.election_id,
        precinct_id,
        ballot_style_id,
    )
    .await
    .map_err(|err| into_internal_error(err))?;

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
