use serde::{ser::SerializeStruct, Serialize};
use uuid::Uuid;

pub struct Election {
    id: Uuid,
}

impl Election {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

impl Serialize for Election {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut election = serializer.serialize_struct("Election", 1)?;
        election.serialize_field("id", &self.id)?;
        election.serialize_field(
            "castBallotsUrl",
            &format!("/api/elections/{}/cast-ballots", self.id),
        )?;
        election.serialize_field(
            "encryptedTallyUrl",
            &format!("/api/elections/{}/encrypted-tally", self.id),
        )?;
        election.serialize_field(
            "decryptedTallyUrl",
            &format!("/api/elections/{}/decrypted-tally", self.id),
        )?;
        election.serialize_field(
            "shuffledBallotsUrl",
            &format!("/api/elections/{}/shuffled-ballots", self.id),
        )?;
        election.end()
    }
}

pub struct CastBallot {
    id: Uuid,
    election_id: Uuid,
}

impl CastBallot {
    pub fn new(id: Uuid, election_id: Uuid) -> Self {
        Self { id, election_id }
    }
}

impl Serialize for CastBallot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut cast_ballot = serializer.serialize_struct("CastBallot", 2)?;
        cast_ballot.serialize_field("id", &self.id)?;
        cast_ballot.serialize_field("electionId", &self.election_id)?;
        cast_ballot.serialize_field(
            "url",
            &format!(
                "/api/elections/{}/cast-ballots/{}",
                self.election_id, self.id
            ),
        )?;
        cast_ballot.end()
    }
}
