use base64::{engine::general_purpose::STANDARD, Engine};
use hmac_sha256::Hash;
#[cfg(feature = "sqlx")]
use sqlx::Type;
use std::{fmt::Display, str::FromStr};
use time::macros::format_description;

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
    pub election_data: Vec<u8>,
    pub election_hash: ElectionHash,
}

impl FromStr for ElectionDefinition {
    type Err = serde_json::Error;

    fn from_str(election_data: &str) -> Result<Self, Self::Err> {
        election_data.as_bytes().try_into()
    }
}

impl TryFrom<&[u8]> for ElectionDefinition {
    type Error = serde_json::Error;

    fn try_from(election_data: &[u8]) -> Result<Self, Self::Error> {
        let election: Election = serde_json::from_slice(election_data)?;

        let mut hasher = Hash::new();
        hasher.update(election_data);
        let election_hash = ElectionHash(hex::encode(hasher.finalize()));

        Ok(Self {
            election,
            election_data: election_data.to_vec(),
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
    Vec<u8>: sqlx::Encode<'q, DB>,
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
        sqlx::postgres::PgTypeInfo::with_name("bytea")
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
        let election_data = STANDARD.encode(&self.election_data);
        election_data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ElectionDefinition {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let election_data = String::deserialize(deserializer)?;
        let election_data = STANDARD
            .decode(election_data.as_str())
            .map_err(serde::de::Error::custom)?;
        election_data
            .as_slice()
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct ElectionHash(String);

impl ElectionHash {
    pub const EXPECTED_BYTE_LENGTH: usize = 32;
    pub const EXPECTED_STRING_LENGTH: usize = Self::EXPECTED_BYTE_LENGTH * 2;

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match hex::decode(&self.0) {
            Ok(bytes) => bytes,
            Err(_) => unreachable!("ElectionHash is always a valid hex string"),
        }
    }

    pub fn to_partial(&self) -> PartialElectionHash {
        PartialElectionHash(self.0[..PartialElectionHash::EXPECTED_STRING_LENGTH].to_string())
    }
}

impl TryFrom<[u8; Self::EXPECTED_BYTE_LENGTH]> for ElectionHash {
    type Error = hex::FromHexError;

    fn try_from(value: [u8; Self::EXPECTED_BYTE_LENGTH]) -> Result<Self, Self::Error> {
        Ok(Self(hex::encode(value)))
    }
}

impl FromStr for ElectionHash {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != Self::EXPECTED_STRING_LENGTH {
            return Err(hex::FromHexError::InvalidStringLength);
        }

        let hex = hex::decode(s)?;
        Ok(Self(hex::encode(hex)))
    }
}

impl std::fmt::Display for ElectionHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[cfg(feature = "sqlx")]
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ElectionHash {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Self(value.as_str()?.to_string()))
    }
}

#[cfg(feature = "sqlx")]
impl<'q, DB> sqlx::Encode<'q, DB> for ElectionHash
where
    String: sqlx::Encode<'q, DB>,
    DB: sqlx::Database,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.0.encode_by_ref(buf)
    }
}

#[cfg(feature = "sqlx")]
impl Type<sqlx::Postgres> for ElectionHash {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("varchar")
    }
}

impl<'de> Deserialize<'de> for ElectionHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let election_hash = String::deserialize(deserializer)?;
        Self::from_str(&election_hash).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ElectionHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartialElectionHash(String);

impl PartialElectionHash {
    pub const EXPECTED_BYTE_LENGTH: usize = 10;
    pub const EXPECTED_STRING_LENGTH: usize = Self::EXPECTED_BYTE_LENGTH * 2;

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match hex::decode(self.0.as_str()) {
            Ok(bytes) => bytes,
            Err(_) => unreachable!("PartialElectionHash is always a valid hex string"),
        }
    }

    pub fn matches_election_hash(&self, election_hash: &ElectionHash) -> bool {
        election_hash.as_str().starts_with(self.as_str())
    }
}

impl TryFrom<[u8; Self::EXPECTED_BYTE_LENGTH]> for PartialElectionHash {
    type Error = hex::FromHexError;

    fn try_from(value: [u8; Self::EXPECTED_BYTE_LENGTH]) -> Result<Self, Self::Error> {
        Ok(Self(hex::encode(value)))
    }
}

impl FromStr for PartialElectionHash {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != PartialElectionHash::EXPECTED_STRING_LENGTH {
            return Err(hex::FromHexError::InvalidStringLength);
        }

        let hex = hex::decode(s)?;
        assert_eq!(hex.len(), PartialElectionHash::EXPECTED_BYTE_LENGTH);
        Ok(Self(hex::encode(hex)))
    }
}

impl std::fmt::Display for PartialElectionHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}…", &self.0)
    }
}

impl<'de> Deserialize<'de> for PartialElectionHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let election_hash = String::deserialize(deserializer)?;
        Self::from_str(&election_hash).map_err(serde::de::Error::custom)
    }
}

impl Serialize for PartialElectionHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

mod election_date {
    use super::*;
    use serde::{de, Deserialize, Deserializer, Serializer};
    use time::format_description;

    const DATE_FORMATTER: &[format_description::FormatItem] =
        format_description!("[year]-[month]-[day]");

    pub fn serialize<S>(date: &time::Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        date.format(DATE_FORMATTER)
            .unwrap_or_default()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<time::Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let date_str = String::deserialize(deserializer)?;
        time::Date::parse(date_str.as_str(), DATE_FORMATTER)
            .map_err(|e| de::Error::custom(format!("invalid date: {e}")))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub title: String,
    #[serde(with = "election_date")]
    pub date: time::Date,
    pub ballot_styles: Vec<BallotStyle>,
    pub precincts: Vec<Precinct>,
    pub districts: Vec<District>,
    pub parties: Vec<Party>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct District {
    pub id: DistrictId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Party {
    pub id: PartyId,
    pub name: String,
    pub full_name: String,
    pub abbrev: String,
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
    pub ballot_style_id: BallotStyleId,
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
