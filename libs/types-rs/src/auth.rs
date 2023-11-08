use serde::{Deserialize, Serialize};

use crate::election::{BallotStyleId, PrecinctId};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemAdministratorUser {
    jurisdiction: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElectionManagerUser {
    jurisdiction: String,
    election_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PollWorkerUser {
    jurisdiction: String,
    election_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardlessVoterUser {
    ballot_style_id: BallotStyleId,
    precinct_id: PrecinctId,
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
}
