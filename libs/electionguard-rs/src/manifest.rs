use serde::{Deserialize, Serialize};
use types_rs::election as vx_election;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Manifest {
    pub election_scope_id: String,
    pub spec_version: String,
    pub r#type: ElectionType,
    pub start_date: String,
    pub end_date: String,
    pub geopolitical_units: Vec<GeopoliticalUnit>,
    pub parties: Vec<Party>,
    pub candidates: Vec<Candidate>,
    pub contests: Vec<Contest>,
    pub ballot_styles: Vec<BallotStyle>,
    pub name: Vec<String>,
    pub contact_information: ContactInformation,
}

impl From<vx_election::Election> for Manifest {
    fn from(value: vx_election::Election) -> Self {
        Self {
            election_scope_id: "TestManifest".to_owned(),
            spec_version: "v2.0.0".to_owned(),
            r#type: if value.title.contains("Primary") {
                ElectionType::Primary
            } else {
                ElectionType::General
            },
            start_date: "start".to_owned(),
            end_date: "end".to_owned(),
            geopolitical_units: value.districts.into_iter().map(Into::into).collect(),
            parties: value.parties.into_iter().map(Into::into).collect(),
            candidates: value
                .contests
                .clone()
                .into_iter()
                .filter_map(|contest| match contest {
                    vx_election::Contest::Candidate(contest) => Some(contest.candidates),
                    vx_election::Contest::YesNo(_) => None,
                })
                .flat_map(|candidates| candidates.into_iter().map(Into::into))
                .collect(),
            contests: value
                .contests
                .into_iter()
                .enumerate()
                .map(|(i, contest)| Contest {
                    sequence_order: i as u32 + 1,
                    ..contest.into()
                })
                .collect(),
            ballot_styles: value.ballot_styles.into_iter().map(Into::into).collect(),
            name: Vec::new(),
            contact_information: ContactInformation::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ElectionType {
    #[serde(rename = "primary")]
    Primary,

    #[serde(rename = "general")]
    General,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ObjectId(pub(crate) String);

impl From<vx_election::DistrictId> for ObjectId {
    fn from(district_id: vx_election::DistrictId) -> Self {
        Self(format!("district-{district_id}"))
    }
}

impl From<vx_election::ContestId> for ObjectId {
    fn from(contest_id: vx_election::ContestId) -> Self {
        Self(format!("contest-{contest_id}"))
    }
}

impl From<vx_election::CandidateId> for ObjectId {
    fn from(candidate_id: vx_election::CandidateId) -> Self {
        Self(format!("cand-{candidate_id}"))
    }
}

impl From<vx_election::PartyId> for ObjectId {
    fn from(party_id: vx_election::PartyId) -> Self {
        Self(format!("party-{party_id}"))
    }
}

impl From<vx_election::BallotStyleId> for ObjectId {
    fn from(ballot_style_id: vx_election::BallotStyleId) -> Self {
        Self(format!("ballot-style-{ballot_style_id}"))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct GeopoliticalUnit {
    pub object_id: ObjectId,
    pub name: String,
    pub r#type: GeopoliticalUnitType,
    pub contact_information: Option<Vec<ContactInformation>>,
}

impl From<vx_election::District> for GeopoliticalUnit {
    fn from(district: vx_election::District) -> Self {
        GeopoliticalUnit {
            object_id: district.id.into(),
            name: district.name,
            r#type: GeopoliticalUnitType::District,
            contact_information: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum GeopoliticalUnitType {
    #[serde(rename = "district")]
    District,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Party {
    pub object_id: ObjectId,
    pub name: String,
    pub abbreviation: String,
    pub color: Option<String>,
    pub logo_uri: Option<String>,
}

impl From<types_rs::election::Party> for Party {
    fn from(party: types_rs::election::Party) -> Self {
        Party {
            object_id: party.id.into(),
            name: party.name,
            abbreviation: party.abbrev,
            color: None,
            logo_uri: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Candidate {
    pub object_id: ObjectId,
    pub name: String,
    pub party_id: Option<ObjectId>,
    pub image_url: Option<String>,
    pub is_write_in: bool,
}

impl From<vx_election::Candidate> for Candidate {
    fn from(candidate: vx_election::Candidate) -> Self {
        Candidate {
            object_id: candidate.id.into(),
            name: candidate.name,
            party_id: candidate
                .party_ids
                .and_then(|party_ids| party_ids.first().cloned())
                .map(Into::into),
            image_url: None,
            is_write_in: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Contest {
    pub object_id: ObjectId,
    pub sequence_order: u32,
    pub electoral_district_id: ObjectId,
    pub vote_variation: VoteVariation,
    pub number_elected: u32,
    pub votes_allowed: u32,
    pub name: String,
    pub ballot_selections: Vec<BallotSelection>,
    pub ballot_title: Option<String>,
    pub ballot_subtitle: Option<String>,
}

impl From<vx_election::Contest> for Contest {
    fn from(contest: vx_election::Contest) -> Self {
        match contest {
            vx_election::Contest::Candidate(contest) => Contest {
                object_id: contest.id.clone().into(),
                sequence_order: 0,
                electoral_district_id: contest.district_id.into(),
                vote_variation: if contest.seats == 1 {
                    VoteVariation::OneOfM
                } else {
                    VoteVariation::NofM
                },
                votes_allowed: contest.seats,
                number_elected: 1,
                name: ObjectId::from(contest.id).0,
                ballot_selections: contest
                    .candidates
                    .into_iter()
                    .enumerate()
                    .map(|(i, candidate)| BallotSelection {
                        sequence_order: i as u32 + 1,
                        ..candidate.into()
                    })
                    .collect(),
                ballot_title: Some(contest.title),
                ballot_subtitle: None,
            },
            vx_election::Contest::YesNo(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BallotSelection {
    pub object_id: ObjectId,
    pub sequence_order: u32,
    pub candidate_id: ObjectId,
}

impl From<vx_election::Candidate> for BallotSelection {
    fn from(value: vx_election::Candidate) -> Self {
        let candidate_id: ObjectId = value.id.into();
        BallotSelection {
            object_id: candidate_id.clone(),
            sequence_order: 0,
            candidate_id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum VoteVariation {
    #[serde(rename = "one_of_m")]
    OneOfM,

    #[serde(rename = "n_of_m")]
    NofM,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct BallotStyle {
    pub object_id: ObjectId,
    pub geopolitical_unit_ids: Vec<ObjectId>,
    pub party_ids: Vec<ObjectId>,
    pub image_uri: Option<String>,
}

impl From<vx_election::BallotStyle> for BallotStyle {
    fn from(ballot_style: vx_election::BallotStyle) -> Self {
        BallotStyle {
            object_id: ballot_style.id.into(),
            geopolitical_unit_ids: ballot_style.districts.into_iter().map(Into::into).collect(),
            party_ids: ballot_style
                .party_id
                .map(|party_id| vec![party_id.into()])
                .unwrap_or_default(),
            image_uri: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ContactInformation {
    pub name: String,
    pub address_line: Vec<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl Default for ContactInformation {
    fn default() -> Self {
        Self {
            name: "contact".to_owned(),
            address_line: Vec::new(),
            email: None,
            phone: None,
        }
    }
}
