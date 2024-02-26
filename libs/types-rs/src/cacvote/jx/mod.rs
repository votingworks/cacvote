use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};

use crate::{
    auth::User,
    cdf::cvr::Cvr,
    election::{BallotStyle, BallotStyleId, ElectionHash, PrecinctId},
};

use super::{ClientId, ServerId};

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub id: ClientId,
    pub server_id: Option<ServerId>,
    pub title: String,
    pub date: time::Date,
    pub ballot_styles: Vec<BallotStyle>,
    pub election_hash: ElectionHash,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

impl Election {
    #[must_use]
    pub const fn new(
        id: ClientId,
        server_id: Option<ServerId>,
        title: String,
        date: time::Date,
        ballot_styles: Vec<BallotStyle>,
        election_hash: ElectionHash,
        created_at: time::OffsetDateTime,
    ) -> Self {
        Self {
            id,
            server_id,
            title,
            date,
            ballot_styles,
            election_hash,
            created_at,
        }
    }

    #[must_use]
    pub fn id(&self) -> &ClientId {
        &self.id
    }

    #[must_use]
    pub fn is_synced(&self) -> bool {
        self.server_id.is_some()
    }

    #[must_use]
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
    #[must_use]
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

    #[must_use]
    pub fn id(&self) -> &ClientId {
        &self.id
    }

    #[must_use]
    pub fn common_access_card_id(&self) -> &str {
        &self.common_access_card_id
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    #[must_use]
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
    election_title: String,
    election_hash: ElectionHash,
    precinct_id: PrecinctId,
    ballot_style_id: BallotStyleId,
    #[serde(with = "time::serde::iso8601")]
    created_at: time::OffsetDateTime,
}

impl Registration {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        id: ClientId,
        server_id: Option<ServerId>,
        display_name: String,
        common_access_card_id: String,
        registration_request_id: ClientId,
        election_title: String,
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
            election_title,
            election_hash,
            precinct_id,
            ballot_style_id,
            created_at,
        }
    }

    #[must_use]
    pub fn id(&self) -> &ClientId {
        &self.id
    }

    #[must_use]
    pub fn is_synced(&self) -> bool {
        self.server_id.is_some()
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        self.display_name.as_str()
    }

    #[must_use]
    pub fn election_title(&self) -> &str {
        &self.election_title
    }

    #[must_use]
    pub fn election_hash(&self) -> &ElectionHash {
        &self.election_hash
    }

    #[must_use]
    pub fn common_access_card_id(&self) -> &str {
        &self.common_access_card_id
    }

    #[must_use]
    pub fn ballot_style_id(&self) -> &BallotStyleId {
        &self.ballot_style_id
    }

    #[must_use]
    pub fn precinct_id(&self) -> &PrecinctId {
        &self.precinct_id
    }

    #[must_use]
    pub fn is_registration_request(&self, registration_request: &RegistrationRequest) -> bool {
        self.registration_request_id == registration_request.id
    }

    #[must_use]
    pub fn created_at(&self) -> &time::OffsetDateTime {
        &self.created_at
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Success {
        common_access_card_id: String,
        display_name: String,
    },
    Failure,
    Error(String),
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrintedBallot {
    pub id: ClientId,
    pub server_id: ServerId,
    pub registration_id: ClientId,
    pub election_id: ClientId,
    pub ballot_style_id: BallotStyleId,
    pub precinct_id: PrecinctId,
    #[serde(with = "Base64Standard")]
    pub cast_vote_record: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub cast_vote_record_signature: Vec<u8>,
    pub verification_status: VerificationStatus,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

impl PrintedBallot {
    #[must_use]
    pub fn election_id(&self) -> &ClientId {
        &self.election_id
    }

    #[must_use]
    pub fn ballot_style_id(&self) -> &BallotStyleId {
        &self.ballot_style_id
    }

    #[must_use]
    pub fn precinct_id(&self) -> &PrecinctId {
        &self.precinct_id
    }

    #[must_use]
    pub fn created_at(&self) -> &time::OffsetDateTime {
        &self.created_at
    }

    /// Returns the cast vote record as a CVR.
    ///
    /// # Errors
    ///
    /// Returns an error if the cast vote record is not valid JSON.
    pub fn cast_vote_record(&self) -> color_eyre::Result<Cvr> {
        let cast_vote_record_json = serde_json::from_slice(&self.cast_vote_record)?;
        Ok(cast_vote_record_json)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScannedBallot {
    pub id: ClientId,
    pub server_id: ServerId,
    pub election_id: ClientId,
    pub precinct_id: PrecinctId,
    pub ballot_style_id: BallotStyleId,
    #[serde(with = "Base64Standard")]
    pub cast_vote_record: Vec<u8>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

impl ScannedBallot {
    #[must_use]
    pub fn created_at(&self) -> &time::OffsetDateTime {
        &self.created_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum SmartcardStatus {
    #[default]
    NoReader,
    NoCard,
    Card,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum AppData {
    #[serde(rename_all = "camelCase")]
    LoggedOut { auth: SmartcardStatus },
    #[serde(rename_all = "camelCase")]
    LoggedIn {
        auth: User,
        app_data: LoggedInAppData,
    },
}

impl Default for AppData {
    fn default() -> Self {
        Self::LoggedOut {
            auth: SmartcardStatus::NoReader,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoggedInAppData {
    pub elections: Vec<Election>,
    pub registration_requests: Vec<RegistrationRequest>,
    pub registrations: Vec<Registration>,
    pub printed_ballots: Vec<PrintedBallot>,
    pub scanned_ballots: Vec<ScannedBallot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRegistrationData {
    pub election_id: ClientId,
    pub registration_request_id: ClientId,
    pub ballot_style_id: BallotStyleId,
    pub precinct_id: PrecinctId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateElectionData {
    pub election_data: String,
    pub return_address: String,
}
