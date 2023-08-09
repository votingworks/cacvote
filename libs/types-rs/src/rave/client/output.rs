use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};

use crate::election::{ElectionDefinition, ElectionHash};
use crate::rave::{ClientId, ServerId};

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    pub common_access_card_id: String,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub server_id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub definition: ElectionDefinition,
    pub election_hash: ElectionHash,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrintedBallot {
    pub server_id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub common_access_card_id: String,
    pub registration_id: ServerId,
    #[serde(with = "Base64Standard")]
    pub cast_vote_record: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub cast_vote_record_signature: Vec<u8>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScannedBallot {
    pub server_id: ServerId,
    pub client_id: ClientId,
    pub machine_id: String,
    pub election_id: ServerId,
    #[serde(with = "Base64Standard")]
    pub cast_vote_record: Vec<u8>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_printed_ballot_serialization() {
        let printed_ballot = PrintedBallot {
            server_id: ServerId::new(),
            client_id: ClientId::new(),
            machine_id: "machine-1".to_string(),
            common_access_card_id: "card-1".to_string(),
            registration_id: ServerId::new(),
            cast_vote_record: vec![1, 2, 3],
            cast_vote_record_signature: vec![4, 5, 6],
            created_at: time::OffsetDateTime::now_utc(),
        };

        let serialized = serde_json::to_string(&printed_ballot).unwrap();
        let expected_cast_vote_record = STANDARD.encode(&printed_ballot.cast_vote_record);

        assert!(
            serialized.contains(&expected_cast_vote_record),
            "serialized: {serialized}",
        );
        assert_eq!(
            serde_json::from_str::<PrintedBallot>(&serialized).unwrap(),
            printed_ballot,
        );
    }
}
