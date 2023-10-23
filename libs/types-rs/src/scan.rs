use serde::{Deserialize, Serialize};

use crate::rave::ClientId;

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScannedBallotStats {
    pub batches: Vec<BatchStats>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchStats {
    pub id: ClientId,
    pub ballot_count: i32,
    pub election_count: i32,
    pub synced_count: i32,
    #[serde(with = "time::serde::iso8601")]
    pub started_at: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601::option")]
    pub ended_at: Option<time::OffsetDateTime>,
}
