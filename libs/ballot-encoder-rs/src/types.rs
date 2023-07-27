use types_rs::cdf::cvr::{Cvr, VxBallotType};

use crate::consts::ELECTION_HASH_HEX_LENGTH;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestMode {
    Test,
    Live,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncodableCvr {
    pub partial_election_hash: String,
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
    pub fn new(partial_election_hash: String, cvr: Cvr, test_mode: TestMode) -> Self {
        Self {
            partial_election_hash: partial_election_hash
                .get(0..ELECTION_HASH_HEX_LENGTH)
                .unwrap_or(&partial_election_hash)
                .to_lowercase(),
            cvr,
            test_mode,
        }
    }
}
