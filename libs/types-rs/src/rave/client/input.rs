use serde::{Deserialize, Serialize};

use crate::cdf::cvr;
use crate::election::ElectionDefinition;
use crate::rave::{ClientId, ServerId};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    pub common_access_card_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub client_id: ClientId,
    pub machine_id: String,
    pub election: ElectionDefinition,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub client_id: ClientId,
    pub machine_id: String,
    pub common_access_card_id: String,
    pub registration_request_id: ClientId,
    pub election_id: ClientId,
    pub precinct_id: String,
    pub ballot_style_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintedBallot {
    pub client_id: ClientId,
    pub machine_id: String,
    pub common_access_card_id: String,
    pub registration_id: ClientId,
    pub cast_vote_record: cvr::Cvr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedBallot {
    pub client_id: ClientId,
    pub machine_id: String,
    pub election_id: ServerId,
    pub cast_vote_record: cvr::Cvr,
}
