use serde::{Deserialize, Serialize};

use crate::election::{BallotStyleId, PrecinctId};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemAdministratorUser {
    jurisdiction: String,
}

impl SystemAdministratorUser {
    #[must_use]
    pub const fn new(jurisdiction: String) -> Self {
        Self { jurisdiction }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElectionManagerUser {
    jurisdiction: String,
    election_hash: String,
}

impl ElectionManagerUser {
    #[must_use]
    pub const fn new(jurisdiction: String, election_hash: String) -> Self {
        Self {
            jurisdiction,
            election_hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PollWorkerUser {
    jurisdiction: String,
    election_hash: String,
}

impl PollWorkerUser {
    #[must_use]
    pub const fn new(jurisdiction: String, election_hash: String) -> Self {
        Self {
            jurisdiction,
            election_hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardlessVoterUser {
    ballot_style_id: BallotStyleId,
    precinct_id: PrecinctId,
}

impl CardlessVoterUser {
    #[must_use]
    pub const fn new(ballot_style_id: BallotStyleId, precinct_id: PrecinctId) -> Self {
        Self {
            ballot_style_id,
            precinct_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RaveAdministratorUser {
    jurisdiction: String,
}

impl RaveAdministratorUser {
    #[must_use]
    pub const fn new(jurisdiction: String) -> Self {
        Self { jurisdiction }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum User {
    #[serde(rename = "system_administrator")]
    SystemAdministrator(SystemAdministratorUser),
    #[serde(rename = "election_manager")]
    ElectionManager(ElectionManagerUser),
    #[serde(rename = "poll_worker")]
    PollWorker(PollWorkerUser),
    #[serde(rename = "cardless_voter")]
    CardlessVoter(CardlessVoterUser),
    #[serde(rename = "rave_admin")]
    RaveAdministrator(RaveAdministratorUser),
}
