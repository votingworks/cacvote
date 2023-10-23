use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::election::{Candidate, ContestId};

pub type VotesDict = HashMap<ContestId, Vote>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Vote {
    Candidate(CandidateVote),
    YesNo(YesNoVote),
}

pub type CandidateVote = Vec<Candidate>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum YesNoVote {
    Empty(),
    One(YesOrNo),
    Both(YesOrNo, YesOrNo),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum YesOrNo {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let votes = VotesDict::new();
        let serialized = serde_json::to_string(&votes).unwrap();
        assert_eq!(serialized, "{}");
    }

    #[test]
    fn test_deserialize() {
        let votes = VotesDict::new();
        let serialized = serde_json::to_string(&votes).unwrap();
        let deserialized: VotesDict = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, votes);
    }
}
