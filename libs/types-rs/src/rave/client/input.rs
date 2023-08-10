use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};

use crate::election::ElectionDefinition;
use crate::rave::ClientId;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

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
    pub definition: ElectionDefinition,
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
    #[serde(with = "Base64Standard")]
    pub common_access_card_certificate: Vec<u8>,
    pub registration_id: ClientId,
    #[serde(with = "Base64Standard")]
    pub cast_vote_record: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub cast_vote_record_signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedBallot {
    pub client_id: ClientId,
    pub machine_id: String,
    pub election_id: ClientId,
    #[serde(with = "Base64Standard")]
    pub cast_vote_record: Vec<u8>,
}
