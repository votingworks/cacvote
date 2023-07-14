use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[repr(transparent)]
// TODO: figure out how to hide the inner value and still work with sqlx
pub struct ServerId(pub Uuid);

impl ServerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[repr(transparent)]
// TODO: figure out how to hide the inner value and still work with sqlx
pub struct ClientId(pub Uuid);

pub mod input {
    use serde::Deserialize;

    use crate::cvr;

    use super::ClientId;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct RegistrationRequest {
        pub client_id: ClientId,
        pub machine_id: String,
        pub common_access_card_id: String,
        pub given_name: String,
        pub family_name: String,
        pub address_line_1: String,
        #[serde(default)]
        pub address_line_2: Option<String>,
        pub city: String,
        pub state: String,
        pub postal_code: String,
        pub state_id: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Election {
        pub client_id: ClientId,
        pub machine_id: String,
        pub election: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Registration {
        pub client_id: ClientId,
        pub machine_id: String,
        pub common_access_card_id: String,
        pub registration_request_id: ClientId,
        pub election_id: ClientId,
        pub precinct_id: String,
        pub ballot_style_id: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Ballot {
        pub client_id: ClientId,
        pub machine_id: String,
        pub common_access_card_id: String,
        pub registration_id: ClientId,
        pub cast_vote_record: cvr::Cvr,
    }
}

pub mod output {
    use serde::Serialize;
    use sqlx::types::Json;

    use crate::cvr;

    use super::{ClientId, ServerId};

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Admin {
        pub common_access_card_id: String,
        #[serde(with = "time::serde::iso8601")]
        pub created_at: time::OffsetDateTime,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct RegistrationRequest {
        pub server_id: ServerId,
        pub client_id: ClientId,
        pub machine_id: String,
        pub common_access_card_id: String,
        pub given_name: String,
        pub family_name: String,
        pub address_line_1: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub address_line_2: Option<String>,
        pub city: String,
        pub state: String,
        pub postal_code: String,
        pub state_id: String,
        #[serde(with = "time::serde::iso8601")]
        pub created_at: time::OffsetDateTime,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Registration {
        pub server_id: ServerId,
        pub client_id: ClientId,
        pub machine_id: String,
        pub common_access_card_id: String,
        pub registration_request_id: ServerId,
        pub election_id: ServerId,
        pub precinct_id: String,
        pub ballot_style_id: String,
        #[serde(with = "time::serde::iso8601")]
        pub created_at: time::OffsetDateTime,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Election {
        pub server_id: ServerId,
        pub client_id: ClientId,
        pub machine_id: String,
        pub election: Json<serde_json::Value>,
        #[serde(with = "time::serde::iso8601")]
        pub created_at: time::OffsetDateTime,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct Ballot {
        pub server_id: ServerId,
        pub client_id: ClientId,
        pub machine_id: String,
        pub common_access_card_id: String,
        pub registration_id: ServerId,
        pub cast_vote_record: cvr::Cvr,
        #[serde(with = "time::serde::iso8601")]
        pub created_at: time::OffsetDateTime,
    }
}
