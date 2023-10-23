use std::str::FromStr;

use types_rs::{
    cdf::cvr::{Cvr, VxBallotType},
    election::PartialElectionHash,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestMode {
    Test,
    Live,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncodableCvr {
    pub partial_election_hash: PartialElectionHash,
    pub cvr: Cvr,
    pub test_mode: TestMode,
}

#[derive(Debug)]
pub struct BallotConfig {
    pub precinct_index: u32,
    pub ballot_style_index: u32,
    pub ballot_type: VxBallotType,
    pub ballot_mode: TestMode,
    pub ballot_id: Option<String>,
}

impl EncodableCvr {
    pub fn new(partial_election_hash: PartialElectionHash, cvr: Cvr, test_mode: TestMode) -> Self {
        Self {
            partial_election_hash: PartialElectionHash::from_str(partial_election_hash.as_str())
                .unwrap_or(partial_election_hash),
            cvr,
            test_mode,
        }
    }
}
