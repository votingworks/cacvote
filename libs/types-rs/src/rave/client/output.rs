use serde::{Deserialize, Serialize};

use crate::cdf::cvr;
use crate::election::ElectionDefinition;
use crate::rave::{ClientId, ServerId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    pub common_access_card_id: String,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub server_id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub election: ElectionDefinition,
    pub election_hash: String,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintedBallot {
    pub server_id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub common_access_card_id: String,
    pub registration_id: ServerId,
    pub cast_vote_record: cvr::Cvr,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedBallot {
    pub server_id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub election_id: ServerId,
    pub cast_vote_record: cvr::Cvr,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}
