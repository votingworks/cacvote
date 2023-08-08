use serde::{Deserialize, Serialize};

use crate::election::{BallotStyle, BallotStyleId, ElectionHash, PrecinctId};

use super::{ClientId, ServerId};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub id: ClientId,
    pub server_id: Option<ServerId>,
    pub title: String,
    pub ballot_styles: Vec<BallotStyle>,
    pub election_hash: ElectionHash,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

impl Election {
    pub const fn new(
        id: ClientId,
        server_id: Option<ServerId>,
        title: String,
        ballot_styles: Vec<BallotStyle>,
        election_hash: ElectionHash,
        created_at: time::OffsetDateTime,
    ) -> Self {
        Self {
            id,
            server_id,
            title,
            ballot_styles,
            election_hash,
            created_at,
        }
    }

    pub fn is_synced(&self) -> bool {
        self.server_id.is_some()
    }

    pub fn created_at(&self) -> &time::OffsetDateTime {
        &self.created_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
    id: ClientId,
    server_id: ServerId,
    common_access_card_id: String,
    display_name: String,
    #[serde(with = "time::serde::iso8601")]
    created_at: time::OffsetDateTime,
}

impl RegistrationRequest {
    pub const fn new(
        id: ClientId,
        server_id: ServerId,
        common_access_card_id: String,
        display_name: String,
        created_at: time::OffsetDateTime,
    ) -> Self {
        Self {
            id,
            server_id,
            common_access_card_id,
            display_name,
            created_at,
        }
    }

    pub fn id(&self) -> &ClientId {
        &self.id
    }

    pub fn common_access_card_id(&self) -> &str {
        &self.common_access_card_id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn created_at(&self) -> &time::OffsetDateTime {
        &self.created_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    id: ClientId,
    server_id: Option<ServerId>,
    display_name: String,
    common_access_card_id: String,
    registration_request_id: ClientId,
    election_hash: ElectionHash,
    precinct_id: PrecinctId,
    ballot_style_id: BallotStyleId,
    #[serde(with = "time::serde::iso8601")]
    created_at: time::OffsetDateTime,
}

impl Registration {
    pub const fn new(
        id: ClientId,
        server_id: Option<ServerId>,
        display_name: String,
        common_access_card_id: String,
        registration_request_id: ClientId,
        election_hash: ElectionHash,
        precinct_id: PrecinctId,
        ballot_style_id: BallotStyleId,
        created_at: time::OffsetDateTime,
    ) -> Self {
        Self {
            id,
            server_id,
            display_name,
            common_access_card_id,
            registration_request_id,
            election_hash,
            precinct_id,
            ballot_style_id,
            created_at,
        }
    }

    pub fn id(&self) -> &ClientId {
        &self.id
    }

    pub fn is_synced(&self) -> bool {
        self.server_id.is_some()
    }

    pub fn display_name(&self) -> &str {
        self.common_access_card_id()
    }

    pub fn election_hash(&self) -> &ElectionHash {
        &self.election_hash
    }

    pub fn common_access_card_id(&self) -> &str {
        &self.common_access_card_id
    }

    pub fn ballot_style_id(&self) -> &BallotStyleId {
        &self.ballot_style_id
    }

    pub fn precinct_id(&self) -> &PrecinctId {
        &self.precinct_id
    }

    pub fn is_registration_request(&self, registration_request: &RegistrationRequest) -> bool {
        self.registration_request_id == registration_request.id
    }

    pub fn created_at(&self) -> &time::OffsetDateTime {
        &self.created_at
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub elections: Vec<Election>,
    pub registration_requests: Vec<RegistrationRequest>,
    pub registrations: Vec<Registration>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRegistrationData {
    pub election_id: ClientId,
    pub registration_request_id: ClientId,
    pub ballot_style_id: BallotStyleId,
    pub precinct_id: PrecinctId,
}
