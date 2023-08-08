use sha256::digest;
#[cfg(feature = "sqlx")]
use sqlx::Type;
use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{ballot_card::BallotSide, geometry::GridUnit, util::idtype};

idtype!(BallotStyleId);
idtype!(CandidateId);
idtype!(ContestId);
idtype!(DistrictId);
idtype!(OptionId);
idtype!(PartyId);
idtype!(PrecinctId);

#[derive(Debug, Clone, PartialEq)]
pub struct ElectionDefinition {
    pub election: Election,
    pub election_data: String,
    pub election_hash: String,
}

impl FromStr for ElectionDefinition {
    type Err = serde_json::Error;

    fn from_str(election_data: &str) -> Result<Self, Self::Err> {
        let election: Election = serde_json::from_str(election_data)?;

        let election_hash = digest(election_data);

        Ok(Self {
            election,
            election_data: election_data.to_string(),
            election_hash,
        })
    }
}

#[cfg(feature = "sqlx")]
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ElectionDefinition
where
    sqlx::types::Json<Self>: sqlx::Decode<'r, sqlx::Postgres>,
{
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let election_data_bytes = value.as_bytes()?;
        serde_json::from_slice(election_data_bytes).map_err(Into::into)
    }
}

#[cfg(feature = "sqlx")]
impl<'q, DB> sqlx::Encode<'q, DB> for ElectionDefinition
where
    String: sqlx::Encode<'q, DB>,
    DB: sqlx::Database,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.election_data.encode_by_ref(buf)
    }
}

#[cfg(feature = "sqlx")]
impl Type<sqlx::Postgres> for ElectionDefinition {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("text")
    }
}

impl ElectionDefinition {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> color_eyre::Result<Self> {
        let election_data = std::fs::read_to_string(path)?;
        Self::from_str(&election_data).map_err(Into::into)
    }
}

impl Serialize for ElectionDefinition {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.election_data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ElectionDefinition {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let election_data = String::deserialize(deserializer)?;
        Self::from_str(&election_data).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub title: String,
    pub ballot_styles: Vec<BallotStyle>,
    pub precincts: Vec<Precinct>,
    pub contests: Vec<Contest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_layouts: Option<Vec<GridLayout>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_thresholds: Option<MarkThresholds>,
}

impl Election {
    pub fn get_contests(&self, ballot_style_id: BallotStyleId) -> Option<Vec<&Contest>> {
        let districts = &self
            .ballot_styles
            .iter()
            .find(|ballot_style| ballot_style.id == ballot_style_id)?
            .districts;
        Some(
            self.contests
                .iter()
                .filter(|contest| districts.contains(contest.district_id()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BallotStyle {
    pub id: BallotStyleId,
    pub precincts: Vec<PrecinctId>,
    pub districts: Vec<DistrictId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<PartyId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Precinct {
    pub id: PrecinctId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Contest {
    #[serde(rename = "candidate")]
    Candidate(CandidateContest),
    #[serde(rename = "yesno")]
    YesNo(YesNoContest),
}

impl Contest {
    pub fn id(&self) -> &ContestId {
        match self {
            Self::Candidate(contest) => &contest.id,
            Self::YesNo(contest) => &contest.id,
        }
    }

    pub fn district_id(&self) -> &DistrictId {
        match self {
            Self::Candidate(contest) => &contest.district_id,
            Self::YesNo(contest) => &contest.district_id,
        }
    }

    pub fn option_ids(&self) -> Vec<OptionId> {
        match self {
            Self::Candidate(contest) => contest
                .candidates
                .iter()
                .map(|candidate| OptionId::from(candidate.id.to_string()))
                .collect(),
            Self::YesNo(contest) => vec![
                contest
                    .yes_option
                    .as_ref()
                    .map_or(OptionId::from("yes".to_string()), |o| o.id.clone()),
                contest
                    .no_option
                    .as_ref()
                    .map_or(OptionId::from("no".to_string()), |o| o.id.clone()),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CandidateContest {
    pub id: ContestId,
    pub district_id: DistrictId,
    pub title: String,
    pub seats: u32,
    pub candidates: Vec<Candidate>,
    pub allow_write_ins: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_id: Option<PartyId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub id: CandidateId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub party_ids: Option<Vec<PartyId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_write_in: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_in_index: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct YesNoContest {
    pub id: ContestId,
    pub district_id: DistrictId,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_option: Option<YesNoOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_option: Option<YesNoOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct YesNoOption {
    pub id: OptionId,
    pub label: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridLayout {
    pub precinct_id: PrecinctId,
    pub ballot_style_id: BallotStyleId,
    pub columns: GridUnit,
    pub rows: GridUnit,
    pub option_bounds_from_target_mark: Outset<GridUnit>,
    pub grid_positions: Vec<GridPosition>,
}

impl GridLayout {
    pub fn write_in_positions(&self) -> Vec<&GridPosition> {
        self.grid_positions
            .iter()
            .filter(|grid_position| matches!(grid_position, GridPosition::WriteIn { .. }))
            .collect()
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Outset<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

/// A position on the ballot grid defined by timing marks and the contest/option
/// for which a mark at this position is a vote for.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum GridPosition {
    /// A pre-defined labeled option on the ballot.
    #[serde(rename_all = "camelCase", rename = "option")]
    Option {
        side: BallotSide,
        column: GridUnit,
        row: GridUnit,
        contest_id: ContestId,
        option_id: OptionId,
    },

    /// A write-in option on the ballot.
    #[serde(rename_all = "camelCase", rename = "write-in")]
    WriteIn {
        side: BallotSide,
        column: GridUnit,
        row: GridUnit,
        contest_id: ContestId,
        write_in_index: u32,
    },
}

impl Display for GridPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Option { option_id, .. } => write!(f, "{option_id}"),

            Self::WriteIn { write_in_index, .. } => {
                write!(f, "Write-In {write_in_index}")
            }
        }
    }
}

impl GridPosition {
    pub fn contest_id(&self) -> ContestId {
        match self {
            Self::Option { contest_id, .. } | Self::WriteIn { contest_id, .. } => {
                contest_id.clone()
            }
        }
    }

    pub fn option_id(&self) -> OptionId {
        match self {
            Self::Option { option_id, .. } => option_id.clone(),
            Self::WriteIn { write_in_index, .. } => {
                OptionId::from(format!("write-in-{write_in_index}"))
            }
        }
    }

    pub const fn location(&self) -> GridLocation {
        match self {
            Self::Option {
                side, column, row, ..
            }
            | Self::WriteIn {
                side, column, row, ..
            } => GridLocation::new(*side, *column, *row),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct GridLocation {
    pub side: BallotSide,
    pub column: GridUnit,
    pub row: GridUnit,
}

impl GridLocation {
    pub const fn new(side: BallotSide, column: GridUnit, row: GridUnit) -> Self {
        Self { side, column, row }
    }
}

/// A value between 0 and 1, inclusive.
///
/// Because this is just a type alias it does not enforce that another type
/// with the same underlying representation is not used.
pub type UnitIntervalValue = f32;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkThresholds {
    pub definite: UnitIntervalValue,
    pub marginal: UnitIntervalValue,
    pub write_in_text_area: Option<UnitIntervalValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_location() {
        let location = GridLocation::new(BallotSide::Front, 1, 2);
        assert_eq!(location.side, BallotSide::Front);
        assert_eq!(location.column, 1);
        assert_eq!(location.row, 2);
    }

    #[test]
    fn test_grid_position() {
        let position = GridPosition::Option {
            side: BallotSide::Front,
            column: 1,
            row: 2,
            contest_id: ContestId::from("contest-1".to_string()),
            option_id: OptionId::from("option-1".to_string()),
        };
        assert_eq!(position.location().side, BallotSide::Front);
        assert_eq!(position.location().column, 1);
        assert_eq!(position.location().row, 2);
    }

    #[test]
    fn test_grid_position_option_serialization() {
        let json = r#"{
            "type": "option",
            "side": "front",
            "column": 1,
            "row": 2,
            "contestId": "contest-1",
            "optionId": "option-1"
        }"#;
        match serde_json::from_str(json).unwrap() {
            GridPosition::Option {
                side,
                column,
                row,
                contest_id,
                option_id,
            } => {
                assert_eq!(side, BallotSide::Front);
                assert_eq!(column, 1);
                assert_eq!(row, 2);
                assert_eq!(contest_id, ContestId::from("contest-1".to_string()));
                assert_eq!(option_id, OptionId::from("option-1".to_string()));
            }
            _ => panic!("expected Option"),
        }
    }

    #[test]
    fn test_grid_position_write_in_serialization() {
        let json = r#"{
            "type": "write-in",
            "side": "front",
            "column": 1,
            "row": 2,
            "contestId": "contest-1",
            "writeInIndex": 3
        }"#;
        match serde_json::from_str(json).unwrap() {
            GridPosition::WriteIn {
                side,
                column,
                row,
                contest_id,
                write_in_index,
            } => {
                assert_eq!(side, BallotSide::Front);
                assert_eq!(column, 1);
                assert_eq!(row, 2);
                assert_eq!(contest_id, ContestId::from("contest-1".to_string()));
                assert_eq!(write_in_index, 3);
            }
            _ => panic!("expected WriteIn"),
        }
    }
}
