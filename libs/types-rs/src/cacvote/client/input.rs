use std::fmt::Debug;

use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};

use crate::cacvote::{ClientId, ServerId};
use crate::election::ElectionDefinition;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Jurisdiction {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    pub machine_id: String,
    pub common_access_card_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
    pub client_id: ClientId,
    pub machine_id: String,
    pub jurisdiction_id: ServerId,
    pub common_access_card_id: String,
    pub given_name: String,
    pub family_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub jurisdiction_id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub definition: ElectionDefinition,
    pub return_address: String,
}

impl Debug for Election {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Election")
            .field("jurisdiction_id", &self.jurisdiction_id)
            .field("client_id", &self.client_id)
            .field("machine_id", &self.machine_id)
            .field("return_address", &self.return_address)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub client_id: ClientId,
    pub machine_id: String,
    pub common_access_card_id: String,
    pub jurisdiction_id: ServerId,
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
