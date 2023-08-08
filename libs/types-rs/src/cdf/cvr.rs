use lazy_static::lazy_static;
use nanoid::nanoid;
use regex::Regex;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// A proper fractional value, represented using fractional or decimal notation.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct FractionalNumber(String);

const FRACTIONAL_NUMBER_PATTERN: &str = "([0-9]+/[1-9]+[0-9]*)|(\\.[0-9]+)";
lazy_static! {
    static ref FRACTIONAL_NUMBER_REGEX: Regex = Regex::new(FRACTIONAL_NUMBER_PATTERN).unwrap();
}

impl std::str::FromStr for FractionalNumber {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if FRACTIONAL_NUMBER_REGEX.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(format!(
                "{} does not match {}",
                s, FRACTIONAL_NUMBER_PATTERN
            ))
        }
    }
}

/// Used in `SelectionPosition::IsAllocable` to indicate whether the
/// `SelectionPosition::NumberVotes` should be allocated to the underlying
/// contest option counter.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum AllocationStatus {
    /// To not allocate votes to the contest option's accumulator.
    #[serde(rename = "no")]
    No,

    /// When the decision to allocate votes is unknown, such as when the adjudication is needed.
    #[serde(rename = "unknown")]
    Unknown,

    /// To allocate votes to the contest option's accumulator.
    #[serde(rename = "yes")]
    Yes,
}

/// Used in CVRSnapshot::Status to identify the status of the CVR.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum CVRStatus {
    /// To indicate that the CVR needs to be adjudicated.
    #[serde(rename = "needs-adjudication")]
    NeedsAdjudication,

    /// Used in conjunction with `CVRSnapshot::OtherStatus` when no other value
    /// in this enumeration applies.
    #[serde(rename = "other")]
    Other,
}

/// Used in `CVRSnapshot::Type` to indicate the type of snapshot.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CVRType {
    /// Has been adjudicated.
    #[serde(rename = "interpreted")]
    Interpreted,

    /// After contest rules applied.
    #[serde(rename = "modified")]
    Modified,

    /// As scanned, no contest rules applied.
    #[serde(rename = "original")]
    #[default]
    Original,
}

/// To identify the version of the CVR specification being used, i.e., version
/// 1.0.0.  This will need to be updated for different versions of the
/// specification.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum CastVoteRecordVersion {
    /// Fixed value for the version of this specification.
    #[serde(rename = "1.0.0")]
    V1_0_0,
}

/// Used in `CVRContestSelection::Status` to identify the status of a contest
/// selection in the CVR.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum ContestSelectionStatus {
    /// To indicate that the contest selection was generated per contest rules.
    #[serde(rename = "generated-rules")]
    GeneratedRules,

    /// To indicate that the contest selection was invalidated by the generating device because of contest rules.
    #[serde(rename = "invalidated-rules")]
    InvalidatedRules,

    /// To indicate that the contest selection was flagged by the generating device for adjudication.
    #[serde(rename = "needs-adjudication")]
    NeedsAdjudication,

    /// Used in conjunction with `CVRContestSelection::OtherStatus` when no
    /// other value in this enumeration applies.
    #[serde(rename = "other")]
    Other,
}

/// Used in `CVRContest::Status` to identify the status of a contest in which
/// contest selection(s) were made.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum ContestStatus {
    /// To indicate that the contest has been invalidated by the generating device because of contest rules.
    #[serde(rename = "invalidated-rules")]
    InvalidatedRules,

    /// For a `CVRContest` with no `SelectionPosition`, i.e. to specify the
    /// position contains no marks or other indications.
    #[serde(rename = "not-indicated")]
    NotIndicated,

    /// Used in conjunction with `CVRContest::OtherStatus` when no other value
    /// in this enumeration applies.
    #[serde(rename = "other")]
    Other,

    /// To indicate that the contest was overvoted.
    #[serde(rename = "overvoted")]
    Overvoted,

    /// To indicate that the contest was undervoted.
    #[serde(rename = "undervoted")]
    Undervoted,
}

/// Used in `Hash::Type` to indicate the type of hash being used for an image
/// file.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum HashType {
    /// To indicate that the MD6 message digest algorithm is being used.
    #[serde(rename = "md6")]
    Md6,

    /// Used in conjunction with Hash::OtherType when no other value in this
    /// enumeration applies.
    #[serde(rename = "other")]
    Other,

    /// To indicate that the SHA 256-bit signature is being used.
    #[serde(rename = "sha-256")]
    Sha256,

    /// To indicate that the SHA 512-bit (32-byte) signature is being used.
    #[serde(rename = "sha-512")]
    Sha512,
}

/// Used in `Code::Type` to indicate the type of code/identifier being used.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum IdentifierType {
    /// To indicate that the identifier is a FIPS code.
    #[serde(rename = "fips")]
    Fips,

    /// To indicate that the identifier is from a local-level scheme, i.e.,
    /// unique to a county or city.
    #[serde(rename = "local-level")]
    LocalLevel,

    /// To indicate that the identifier is from a national-level scheme other
    /// than FIPS or OCD-ID.
    #[serde(rename = "national-level")]
    NationalLevel,

    /// To indicate that the identifier is from the OCD-ID scheme.
    #[serde(rename = "ocd-id")]
    OcdId,

    /// Used in conjunction with Code::OtherType when no other value in this
    /// enumeration applies.
    #[serde(rename = "other")]
    Other,

    /// To indicate that the identifier is from a state-level scheme, i.e.,
    /// unique to a particular state.
    #[serde(rename = "state-level")]
    StateLevel,
}

/// Used in SelectionPosition::HasIndication to identify whether a selection
/// indication is present.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum IndicationStatus {
    /// There is no selection indication.
    #[serde(rename = "no")]
    No,

    /// It is unknown whether there is a selection indication, e.g., used for ambiguous marks.
    #[serde(rename = "unknown")]
    #[default]
    Unknown,

    /// There is a selection indication present.
    #[serde(rename = "yes")]
    Yes,
}

/// Used in `SelectionPosition::Status` to identify the status of a selection
/// indication.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum PositionStatus {
    /// Used if the indication was adjudicated.
    #[serde(rename = "adjudicated")]
    Adjudicated,

    /// Used if the indication was generated by the creating device per contest
    /// rules.
    #[serde(rename = "generated-rules")]
    GeneratedRules,

    /// Used if the indication was invalidated by the creating device because of
    /// contest rules.
    #[serde(rename = "invalidated-rules")]
    InvalidatedRules,

    /// Used in conjunction with `SelectionPosition::OtherStatus` when no other value in this enumeration applies.
    #[serde(rename = "other")]
    Other,
}

/// Used in `CastVoteRecordReport::ReportType` to indicate the type of the CVR report.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum ReportType {
    /// To indicate that the report contains adjudications.
    #[serde(rename = "adjudicated")]
    Adjudicated,

    /// To indicate that the report is an aggregation of device reports.
    #[serde(rename = "aggregated")]
    Aggregated,

    /// To indicate that the report is an export from a device such as a
    /// scanner.
    #[serde(rename = "originating-device-export")]
    OriginatingDeviceExport,

    /// Used in conjunction with `CastVoteRecordReport::OtherReportType` when no
    /// other value in this enumeration applies.
    #[serde(rename = "other")]
    Other,

    /// To indicate that the report is the result of a ranked choice voting
    /// round.
    #[serde(rename = "rcv-round")]
    RcvRound,
}

/// Used in `GpUnit::Type` to indicate a type of political geography.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum ReportingUnitType {
    /// To indicate a combined precinct.
    #[serde(rename = "combined-precinct")]
    CombinedPrecinct,

    /// Used in conjunction with `GpUnit::OtherType` when no other value in this
    /// enumeration applies.
    #[serde(rename = "other")]
    Other,

    /// To indicate a polling place.
    #[serde(rename = "polling-place")]
    PollingPlace,

    /// To indicate a precinct.
    #[serde(rename = "precinct")]
    Precinct,

    /// To indicate a split-precinct.
    #[serde(rename = "split-precinct")]
    SplitPrecinct,

    /// To indicate a vote-center.
    #[serde(rename = "vote-center")]
    VoteCenter,
}

/// Used in `Contest::VoteVariation` to indicate the vote variation (vote
/// method) used to tabulate the contest.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum VoteVariation {
    /// To indicate approval voting.
    #[serde(rename = "approval")]
    Approval,

    /// To indicate the borda count method.
    #[serde(rename = "borda")]
    Borda,

    /// To indicate cumulative voting.
    #[serde(rename = "cumulative")]
    Cumulative,

    /// To indicate majority voting.
    #[serde(rename = "majority")]
    Majority,

    /// To indicate the N of M voting method.
    #[serde(rename = "n-of-m")]
    NOfM,

    /// Used in conjunction with `Contest::OtherVoteVariation` when no other
    /// value in this enumeration applies.
    #[serde(rename = "other")]
    Other,

    /// To indicate plurality voting.
    #[serde(rename = "plurality")]
    Plurality,

    /// To indicate proportional voting.
    #[serde(rename = "proportional")]
    Proportional,

    /// To indicate range voting.
    #[serde(rename = "range")]
    Range,

    /// To indicate Ranked Choice Voting (RCV).
    #[serde(rename = "rcv")]
    Rcv,

    /// To indicate the super majority voting method.
    #[serde(rename = "super-majority")]
    SuperMajority,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum AnnotationObjectType {
    #[serde(rename = "CVR.Annotation")]
    #[default]
    Annotation,
}

/// Annotation is used to record annotations made by one or more adjudicators.
/// `CVRSnapshot` includes `Annotation`.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct Annotation {
    #[serde(rename = "@type")]
    pub object_type: AnnotationObjectType,

    /// The name(s) of the adjudicator(s).
    #[serde(rename = "AdjudicatorName", skip_serializing_if = "Option::is_none")]
    pub adjudicator_name: Option<Vec<String>>,

    /// A message created by the adjudicator(s).
    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<Vec<String>>,

    /// The date and time of the annotation.
    #[serde(
        rename = "TimeStamp",
        with = "time::serde::iso8601::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub time_stamp: Option<OffsetDateTime>,
}

/// `BallotMeasureContest` is a subclass of `Contest` and is used to identify
/// the type of contest as involving one or more ballot measures. It inherits
/// attributes from `Contest`.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct BallotMeasureContest {
    #[serde(rename = "@id")]
    pub id: String,

    /// An abbreviation associated with the contest.
    #[serde(rename = "Abbreviation", skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,

    /// A code or identifier used for this contest.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Identifies the contest selections in the contest.
    #[serde(rename = "ContestSelection")]
    pub contest_selection: Vec<AnyContestSelection>,

    /// Title or name of the contest, e.g., "Governor" or "Question on Legalization of Gambling".
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// If `VoteVariation` is 'other', the vote variation for this contest.
    #[serde(rename = "OtherVoteVariation", skip_serializing_if = "Option::is_none")]
    pub other_vote_variation: Option<String>,

    /// The vote variation for this contest, from the `VoteVariation`
    /// enumeration.
    #[serde(rename = "VoteVariation", skip_serializing_if = "Option::is_none")]
    pub vote_variation: Option<VoteVariation>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum BallotMeasureSelectionObjectType {
    #[serde(rename = "CVR.BallotMeasureSelection")]
    #[default]
    BallotMeasure,
}

/// `BallotMeasureSelection` is a subclass of `ContestSelection` and is used for
/// ballot measures.  The voter's selected response to the contest selection
/// (e.g., "yes" or "no") may be in English or other languages as utilized on
/// the voter's ballot.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct BallotMeasureSelection {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: BallotMeasureSelectionObjectType,

    /// Code used to identify the contest selection.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// The voter's selection, i.e., 'yes' or 'no', in English or in other languages as utilized on the voter's ballot.
    #[serde(rename = "Selection")]
    pub selection: String,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CVRObjectType {
    #[serde(rename = "CVR.CVR")]
    #[default]
    Cvr,
}

/// CVR constitutes a cast vote record, generated by a ballot scanning device, containing indications of contests and contest options chosen by the voter, as well as other information for auditing and annotation purposes. Each sheet of a multi-page paper ballot is represented by an individual CVR, e.g., if all sheets of a 5-sheet ballot are scanned, 5 CVRs will be created.  CastVoteRecordReport includes multiple instances of CVR as applicable.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct Cvr {
    #[serde(rename = "@type")]
    pub object_type: CVRObjectType,

    /// A unique identifier for this CVR, used to link the CVR with the corresponding audit record, e.g., a paper ballot.  This identifier may be impressed on the corresponding audit record as it is scanned, or otherwise associated with the corresponding ballot.
    #[serde(rename = "BallotAuditId", skip_serializing_if = "Option::is_none")]
    pub ballot_audit_id: Option<String>,

    /// An image of the ballot sheet created by the scanning device.
    #[serde(rename = "BallotImage", skip_serializing_if = "Option::is_none")]
    pub ballot_image: Option<Vec<ImageData>>,

    /// A unique identifier for the ballot (or sheet of a multi-sheet ballot) that this CVR represents, used if ballots are pre-marked with unique identifiers.  If provided, this number would be the same on all CVRs that represent individual sheets from the same multi-sheet ballot.  This identifier is not the same as one that may be impressed on the corresponding ballot as it is scanned or otherwise associated with the corresponding ballot; see the BallotAuditId attribute.
    #[serde(rename = "BallotPrePrintedId", skip_serializing_if = "Option::is_none")]
    pub ballot_pre_printed_id: Option<String>,

    /// A unique number for the ballot (or sheet of a multi-sheet ballot) that this CVR represents, used if ballots are pre-marked with unique numbers.  If provided, this number would be the same on all CVRs that represent individual sheets from the same multi-sheet ballot.  This number is not the same as one that may be impressed on the corresponding ballot as it is scanned or otherwise associated with the corresponding ballot; see the BallotAuditId attribute.
    #[serde(rename = "BallotSheetId", skip_serializing_if = "Option::is_none")]
    pub ballot_sheet_id: Option<String>,

    /// An identifier of the ballot style associated with the corresponding ballot.
    #[serde(rename = "BallotStyleId", skip_serializing_if = "Option::is_none")]
    pub ballot_style_id: Option<String>,

    /// Identifies the smallest unit of geography associated with the corresponding ballot, typically a precinct or split-precinct.
    #[serde(rename = "BallotStyleUnitId", skip_serializing_if = "Option::is_none")]
    pub ballot_style_unit_id: Option<String>,

    /// The identifier for the batch that includes this CVR.
    #[serde(rename = "BatchId", skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,

    /// The sequence number of the corresponding paper ballot within a batch.
    #[serde(rename = "BatchSequenceId", skip_serializing_if = "Option::is_none")]
    pub batch_sequence_id: Option<i64>,

    /// Identifies the repeatable portion of the CVR that links to contest selections and related information.
    #[serde(rename = "CVRSnapshot")]
    pub cvr_snapshot: Vec<CVRSnapshot>,

    /// Identifies the device that created the CVR.
    #[serde(rename = "CreatingDeviceId", skip_serializing_if = "Option::is_none")]
    pub creating_device_id: Option<String>,

    /// Identifies the snapshot that is currently tabulatable.
    #[serde(rename = "CurrentSnapshotId")]
    pub current_snapshot_id: String,

    /// Used to identify an election with which the CVR is associated.
    #[serde(rename = "ElectionId")]
    pub election_id: String,

    /// Identifies the party associated with a CVR, typically for partisan primaries.
    #[serde(rename = "PartyIds", skip_serializing_if = "Option::is_none")]
    pub party_ids: Option<Vec<String>>,

    /// The sequence number for this CVR. This represents the ordinal number that this CVR was processed by the tabulating device.
    #[serde(rename = "UniqueId", skip_serializing_if = "Option::is_none")]
    pub unique_id: Option<String>,

    /// Indicates whether the ballot is an absentee or precinct ballot.
    #[serde(rename = "vxBallotType")]
    pub vx_ballot_type: VxBallotType,
}

impl Default for Cvr {
    fn default() -> Self {
        Self {
            object_type: CVRObjectType::Cvr,
            ballot_audit_id: None,
            ballot_image: None,
            ballot_pre_printed_id: None,
            ballot_sheet_id: None,
            ballot_style_id: None,
            ballot_style_unit_id: None,
            batch_id: None,
            batch_sequence_id: None,
            cvr_snapshot: vec![],
            creating_device_id: None,
            current_snapshot_id: "".to_string(),
            election_id: "".to_string(),
            party_ids: None,
            unique_id: None,
            vx_ballot_type: VxBallotType::Precinct,
        }
    }
}

/// Used in `CVR::vxBallotType` to indicate whether the ballot is an absentee or
/// precinct ballot.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub enum VxBallotType {
    #[serde(rename = "precinct")]
    Precinct,

    #[serde(rename = "absentee")]
    Absentee,

    #[serde(rename = "provisional")]
    Provisional,
}

impl VxBallotType {
    pub fn max() -> u32 {
        // updating this value is a breaking change
        2u32.pow(4) - 1
    }
}

impl From<VxBallotType> for u32 {
    fn from(vx_ballot_type: VxBallotType) -> Self {
        match vx_ballot_type {
            VxBallotType::Precinct => 0,
            VxBallotType::Absentee => 1,
            VxBallotType::Provisional => 2,
        }
    }
}

impl From<u32> for VxBallotType {
    fn from(vx_ballot_type: u32) -> Self {
        match vx_ballot_type {
            0 => VxBallotType::Precinct,
            1 => VxBallotType::Absentee,
            2 => VxBallotType::Provisional,
            _ => panic!("Invalid VxBallotType"),
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CVRContestObjectType {
    #[serde(rename = "CVR.CVRContest")]
    #[default]
    CVRContest,
}

/// `CVRContest` class is included by `CVRSnapshot` for each contest on the
/// ballot that was voted, that is, whose contest options contain indications
/// that may constitute a vote. CVRContest includes CVRContestSelection for each
/// contest option in the contest containing an indication or write-in.
/// CVRSnapshot can also include CVRContest for every contest on the ballot
/// regardless of whether any of the contest options contain an indication, for
/// cases where the CVR must include all contests that appeared on the ballot.
/// CVRContest attributes are for including summary information about the
/// contest.  Overvotes plus Undervotes plus TotalVotes must equal the number
/// of votes allowable in the contest, e.g., in a &quot;chose 3 of 5&quot;
/// contest in which the voter chooses only 2, then Overvotes = 0, Undervotes =
/// 1, and TotalVotes = 2, which adds up to the number of votes allowable = 3.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct CVRContest {
    #[serde(rename = "@type")]
    pub object_type: CVRContestObjectType,

    /// Used to include information about a contest selection in the contest, including the associated indication(s).
    #[serde(
        rename = "CVRContestSelection",
        skip_serializing_if = "Option::is_none"
    )]
    pub cvr_contest_selection: Option<Vec<CVRContestSelection>>,

    /// Used to link to an instance of Contest specific to the contest at hand, for the purpose of specifying information about the contest such as its contest identifier.
    #[serde(rename = "ContestId")]
    pub contest_id: String,

    /// Used when Status is 'other' to include a user-defined status.
    #[serde(rename = "OtherStatus", skip_serializing_if = "Option::is_none")]
    pub other_status: Option<String>,

    /// The number of votes lost due to overvoting.
    #[serde(rename = "Overvotes", skip_serializing_if = "Option::is_none")]
    pub overvotes: Option<i64>,

    /// Used to indicate the number of possible contest selections in the contest.
    #[serde(rename = "Selections", skip_serializing_if = "Option::is_none")]
    pub selections: Option<i64>,

    /// The status of the contest, e.g., overvoted, undervoted, from the
    /// `ContestStatus` enumeration.  If no values apply, use 'other' and
    /// include a user-defined status in `OtherStatus.`
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<ContestStatus>>,

    /// The number of votes lost due to undervoting.
    #[serde(rename = "Undervotes", skip_serializing_if = "Option::is_none")]
    pub undervotes: Option<i64>,

    /// The total number of write-ins in the contest.
    #[serde(rename = "WriteIns", skip_serializing_if = "Option::is_none")]
    pub write_ins: Option<i64>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CVRContestSelectionObjectType {
    #[serde(rename = "CVR.CVRContestSelection")]
    #[default]
    CVRContestSelection,
}

/// `CVRContestSelection` is used to link a contest option containing an
/// indication with information about the indication, such as whether a mark
/// constitutes a countable vote, or whether a mark is determined to be
/// marginal, etc.  `CVRContest` includes an instance of `CVRContestSelection`
/// when an indication for the selection is present, and `CVRContestSelection`
/// then includes `SelectionPosition` for each indication present. To tie the
/// indication to the specific contest selection, `CVRContestSelection` links to
/// an instance of `ContestSelection` that has previously been included by
/// `Contest.`    Since multiple indications per contest option are possible for
/// some voting methods, `CVRContestSelection` can include multiple instances of
/// SelectionPosition, one per indication. `CVRContestSelection` can also be
/// used for the purpose of including, in the CVR, all contest options in the
/// contest regardless of whether indications are present.  In this case,
/// `CVRContestSelection` would not include `SelectionPosition` if no indication
/// is present but would link to the appropriate instance of `ContestSelection.`
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct CVRContestSelection {
    #[serde(rename = "@type")]
    pub object_type: CVRContestSelectionObjectType,

    /// Used to link to an instance of a contest selection that was previously
    /// included by Contest.
    #[serde(rename = "ContestSelectionId", skip_serializing_if = "Option::is_none")]
    pub contest_selection_id: Option<String>,

    /// Used to include the ordinal position of the contest option as it
    /// appeared on the ballot.
    #[serde(rename = "OptionPosition", skip_serializing_if = "Option::is_none")]
    pub option_position: Option<i64>,

    /// Used when Status is 'other' to include a user-defined status.
    #[serde(rename = "OtherStatus", skip_serializing_if = "Option::is_none")]
    pub other_status: Option<String>,

    /// For the RCV voting variation, the rank chosen by the voter, for when a
    /// contest selection can represent a ranking.
    #[serde(rename = "Rank", skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,

    /// Used to include further information about the indication/mark associated
    /// with the contest selection.  Depending on the voting method, multiple
    /// indications/marks per selection may be possible.
    #[serde(rename = "SelectionPosition")]
    pub selection_position: Vec<SelectionPosition>,

    /// Contains the status of the contest selection, e.g., 'needs-adjudication'
    /// for a contest requiring adjudication, using values from the
    /// `ContestSelectionStatus` enumeration.  If no values apply, use 'other' and
    /// include a user-defined status in OtherStatus.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<ContestSelectionStatus>>,

    /// For cumulative or range and other similar voting variations, contains
    /// the total proper fractional number of votes across all
    /// indications/marks.
    #[serde(
        rename = "TotalFractionalVotes",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_fractional_votes: Option<FractionalNumber>,

    /// For cumulative or range and other similar voting variations, contains
    /// the total number of votes across all indications/marks.
    #[serde(rename = "TotalNumberVotes", skip_serializing_if = "Option::is_none")]
    pub total_number_votes: Option<i64>,
}

/// `CVRSnapshot` contains a version of the contest selections for a CVR; there
/// can be multiple versions of `CVRSnapshot` within the same CVR.  Type
/// specifies the type of the snapshot, i.e., whether interpreted by the scanner
/// according to contest rules, modified as a result of adjudication, or the
/// original, that is, the version initially scanned before contest rules are
/// applied.  CVR includes `CVRSnapshot.Other` attributes are repeated in each
/// `CVRSnapshot` because they may differ across snapshots, e.g., the contests
/// could be different as well as other status.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CVRSnapshotObjectType {
    #[serde(rename = "CVR.CVRSnapshot")]
    #[default]
    CVRSnapshot,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct CVRSnapshot {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: CVRSnapshotObjectType,

    /// Used to include an annotation associated with the CVR snapshot.
    #[serde(rename = "Annotation", skip_serializing_if = "Option::is_none")]
    pub annotation: Option<Vec<Annotation>>,

    /// Identifies the contests in the CVR.
    #[serde(rename = "CVRContest", skip_serializing_if = "Option::is_none")]
    pub cvr_contest: Option<Vec<CVRContest>>,

    /// When Status is 'other', contains the ballot status.
    #[serde(rename = "OtherStatus", skip_serializing_if = "Option::is_none")]
    pub other_status: Option<String>,

    /// The status of the CVR.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<CVRStatus>>,

    /// The type of the snapshot, e.g., original.
    #[serde(rename = "Type")]
    pub snapshot_type: CVRType,
}

impl Default for CVRSnapshot {
    fn default() -> Self {
        Self {
            id: nanoid!(),
            object_type: CVRSnapshotObjectType::default(),
            annotation: None,
            cvr_contest: None,
            other_status: None,
            status: None,
            snapshot_type: CVRType::default(),
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CVRWriteInObjectType {
    #[serde(rename = "CVR.CVRWriteIn")]
    #[default]
    CVRWriteIn,
}

/// `CVRWriteIn` is used when the contest selection is a write-in. It has
/// attributes for the image or text of the write-in.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct CVRWriteIn {
    #[serde(rename = "@type")]
    pub object_type: CVRWriteInObjectType,

    /// Used for the text of the write-in, typically present when the CVR has
    /// been created by electronic ballot marking equipment.
    #[serde(rename = "Text", skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Used for an image of the write-in, typically made by a scanner when
    /// scanning a paper ballot.
    #[serde(rename = "WriteInImage", skip_serializing_if = "Option::is_none")]
    pub write_in_image: Option<ImageData>,
}

/// Candidate identifies a candidate in a contest on the voter's ballot.
/// `Election` includes instances of `Candidate` for each candidate in a contest;
/// typically, only those candidates who received votes would be included.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CandidateObjectType {
    #[serde(rename = "CVR.Candidate")]
    #[default]
    Candidate,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct Candidate {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: CandidateObjectType,

    /// A code or identifier associated with the candidate.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Candidate's name as listed on the ballot.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The party associated with the candidate.
    #[serde(rename = "PartyId", skip_serializing_if = "Option::is_none")]
    pub party_id: Option<String>,
}

/// `CandidateContest` is a subclass of `Contest` and is used to identify the type
/// of contest as involving one or more candidates. It inherits attributes from
/// `Contest.`
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CandidateContestObjectType {
    #[serde(rename = "CVR.CandidateContest")]
    #[default]
    CandidateContest,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct CandidateContest {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: CandidateContestObjectType,

    /// An abbreviation associated with the contest.
    #[serde(rename = "Abbreviation", skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,

    /// A code or identifier used for this contest.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Identifies the contest selections in the contest.
    #[serde(rename = "ContestSelection")]
    pub contest_selection: Vec<AnyContestSelection>,

    /// Title or name of the contest, e.g., "Governor" or "Question on Legalization of Gambling".
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The number of candidates to be elected in the contest.
    #[serde(rename = "NumberElected", skip_serializing_if = "Option::is_none")]
    pub number_elected: Option<i64>,

    /// If VoteVariation is 'other', the vote variation for this contest.
    #[serde(rename = "OtherVoteVariation", skip_serializing_if = "Option::is_none")]
    pub other_vote_variation: Option<String>,

    /// The party associated with the contest, if a partisan primary.
    #[serde(rename = "PrimaryPartyId", skip_serializing_if = "Option::is_none")]
    pub primary_party_id: Option<String>,

    /// The vote variation for this contest, from the VoteVariation enumeration.
    #[serde(rename = "VoteVariation", skip_serializing_if = "Option::is_none")]
    pub vote_variation: Option<VoteVariation>,

    /// The number of votes allowed in the contest, e.g., 3 for a 'choose 3 of 5 candidates' contest.
    #[serde(rename = "VotesAllowed", skip_serializing_if = "Option::is_none")]
    pub votes_allowed: Option<i64>,
}

/// `CandidateSelection` is a subclass of `ContestSelection` and is used for
/// candidates, including for write-in candidates.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CandidateSelectionObjectType {
    #[serde(rename = "CVR.CandidateSelection")]
    #[default]
    CandidateSelection,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct CandidateSelection {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: CandidateSelectionObjectType,

    /// The candidate associated with the contest selection. For contests involving a ticket of multiple candidates, an ordered list of candidates as they appeared on the ballot would be created.
    #[serde(rename = "CandidateIds", skip_serializing_if = "Option::is_none")]
    pub candidate_ids: Option<Vec<String>>,

    /// Code used to identify the contest selection.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// A flag to indicate if the candidate selection is associated with a write-in.
    #[serde(rename = "IsWriteIn", skip_serializing_if = "Option::is_none")]
    pub is_write_in: Option<bool>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CastVoteRecordReportObjectType {
    #[serde(rename = "CVR.CastVoteRecordReport")]
    #[default]
    CastVoteRecordReport,
}

/// The root class/element; attributes pertain to the status and format of the report and when created. CastVoteRecordReport includes multiple instances of CVR, one per CVR or sheet of a multi-page cast vote record.  CastVoteRecordReport also includes multiple instances of Contest, typically only for those contests that were voted so as to reduce file size.  The Contest instances are later referenced by other classes to link them to contest options that were voted and the indication(s)/mark(s) made.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct CastVoteRecordReport {
    #[serde(rename = "@type")]
    pub object_type: CastVoteRecordReportObjectType,

    /// Used to include instances of CVR classes, one per cast vote record in the report.
    #[serde(rename = "CVR", skip_serializing_if = "Option::is_none")]
    pub cvr: Option<Vec<Cvr>>,

    /// Used to include the election(s) associated with the CVRs.
    #[serde(rename = "Election")]
    pub election: Vec<Election>,

    /// Identifies the time that the election report was created.
    #[serde(rename = "GeneratedDate", with = "time::serde::iso8601")]
    pub generated_date: OffsetDateTime,

    /// Used to include the political geography, i.e., location, for where the cast vote record report was created and for linking cast vote records to their corresponding precinct or split (or otherwise smallest unit).
    #[serde(rename = "GpUnit")]
    pub gp_unit: Vec<GpUnit>,

    /// Notes that can be added as appropriate, presumably by an adjudicator.
    #[serde(rename = "Notes", skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// If ReportType is 'other', this contains the report type.
    #[serde(rename = "OtherReportType", skip_serializing_if = "Option::is_none")]
    pub other_report_type: Option<String>,

    /// The party associated with the ballot sheet for a partisan primary.
    #[serde(rename = "Party", skip_serializing_if = "Option::is_none")]
    pub party: Option<Vec<Party>>,

    /// Identifies the device used to create the CVR report.
    #[serde(rename = "ReportGeneratingDeviceIds")]
    pub report_generating_device_ids: Vec<String>,

    /// The type of report, using the ReportType enumeration.
    #[serde(rename = "ReportType", skip_serializing_if = "Option::is_none")]
    pub report_type: Option<Vec<ReportType>>,

    /// The device creating the report.  The reporting device need not necessarily be the creating device, i.e., for an aggregated report, the reporting device could be an EMS used to aggregate and tabulate cast vote records.
    #[serde(rename = "ReportingDevice")]
    pub reporting_device: Vec<ReportingDevice>,

    /// The version of the CVR specification being used (1.0).
    #[serde(rename = "Version")]
    pub version: CastVoteRecordVersion,

    /// List of scanner batches with metadata.
    #[serde(rename = "vxBatch")]
    vx_batch: Vec<VxBatch>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum VxBatchObjectType {
    #[serde(rename = "CVR.vxBatch")]
    #[default]
    VxBatch,
}

/// Entity containing metadata about a scanned batch. Cast vote records link to batches via CVR::BatchId.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct VxBatch {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: VxBatchObjectType,

    /// A human readable label for the batch.
    #[serde(rename = "BatchLabel")]
    batch_label: String,

    /// The ordinal number of the batch in the tabulator's sequence of batches in a given election.
    #[serde(rename = "SequenceId")]
    sequence_id: u64,

    /// The start time of the batch. On a precinct scanner, the start time is when the polls are opened or voting is resumed. On a central scanner, the start time is when the user initiates scanning a batch.
    #[serde(rename = "StartTime", with = "time::serde::iso8601")]
    start_time: OffsetDateTime,

    /// The end time of the batch. On a precinct scanner, the end time is when the polls are closed or voting is paused. On a central scanner, the end time is when a batch scan is complete
    #[serde(
        rename = "EndTime",
        default,
        skip_serializing_if = "Option::is_none",
        with = "time::serde::iso8601::option"
    )]
    end_time: Option<OffsetDateTime>,

    /// The number of sheets included in a batch.
    #[serde(rename = "NumberSheets")]
    number_sheets: u64,

    /// The tabulator that created the batch.
    #[serde(rename = "CreatingDeviceId")]
    creating_device_id: String,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum CodeObjectType {
    #[serde(rename = "CVR.Code")]
    #[default]
    Code,
}

/// Code is used in Election, GpUnit, Contest, Candidate, and Party to identify an associated code as well as the type of code.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct Code {
    #[serde(rename = "@type")]
    pub object_type: CodeObjectType,

    /// A label associated with the code, used as needed.
    #[serde(rename = "Label", skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// If Type is 'other', the type of code.
    #[serde(rename = "OtherType", skip_serializing_if = "Option::is_none")]
    pub other_type: Option<String>,

    /// Used to indicate the type of code, from the IdentifierType enumeration.
    #[serde(rename = "Type")]
    pub code_type: IdentifierType,

    /// The value of the code, i.e., the identifier.
    #[serde(rename = "Value")]
    pub value: String,
}

/// Contest represents a contest on the ballot. CastVoteRecordReport initially includes an instance of Contest for each contest on the ballot.  Other classes can subsequently reference the instances as necessary to link together items on the cast vote record, such as a contest, its voted contest selection(s), and the mark(s) associated with the selection(s).
///
/// Contest has three subclasses, each used for a specific type of contest:   These subclasses inherit Contest's attributes.
///
/// PartyContest - used for straight party contests,
///
/// BallotMeasureContest - used for contests, and
///
/// CandidateContest - used for candidate contests.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct Contest {
    #[serde(rename = "@id")]
    pub id: String,

    /// An abbreviation associated with the contest.
    #[serde(rename = "Abbreviation", skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,

    /// A code or identifier used for this contest.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Identifies the contest selections in the contest.
    #[serde(rename = "ContestSelection")]
    pub contest_selection: Vec<AnyContestSelection>,

    /// Title or name of the contest, e.g., "Governor" or "Question on Legalization of Gambling".
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// If VoteVariation is 'other', the vote variation for this contest.
    #[serde(rename = "OtherVoteVariation", skip_serializing_if = "Option::is_none")]
    pub other_vote_variation: Option<String>,

    /// The vote variation for this contest, from the VoteVariation enumeration.
    #[serde(rename = "VoteVariation", skip_serializing_if = "Option::is_none")]
    pub vote_variation: Option<VoteVariation>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
#[serde(tag = "@type")]
pub enum AnyContestSelection {
    #[serde(rename = "CVR.ContestSelection")]
    Contest(ContestSelection),
    #[serde(rename = "CVR.PartySelection")]
    Party(PartySelection),
    #[serde(rename = "CVR.BallotMeasureSelection")]
    BallotMeasure(BallotMeasureSelection),
    #[serde(rename = "CVR.CandidateSelection")]
    Candidate(CandidateSelection),
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
#[serde(tag = "@type")]
pub enum AnyContest {
    #[serde(rename = "CVR.Contest")]
    Contest(Contest),
    #[serde(rename = "CVR.PartyContest")]
    PartyContest(PartyContest),
    #[serde(rename = "CVR.BallotMeasureContest")]
    BallotMeasureContest(BallotMeasureContest),
    #[serde(rename = "CVR.CandidateContest")]
    CandidateContest(CandidateContest),
}

/// ContestSelection represents a contest selection in a contest.  Contest can include an instance of ContestSelection for each contest selection in the contest or, as desired, all contest selections.
///
/// ContestSelection has three subclasses, each used for a specific type of contest selection:
///
/// BallotMeasureSelection - used for ballot measures,
///
/// CandidateSelection - used for candidate selections, and
///
/// PartySelection - used for straight party selections.
///
/// Instances of CVRContestSelection subsequently link to the contest selections as needed so as to tie together the contest, the contest selection, and the mark(s) made for the contest selection.
///
/// ContestSelection contains one attribute, Code, that can be used to identify the contest selection and thereby eliminate the need to identify it using the subclasses.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum ContestSelectionObjectType {
    #[serde(rename = "CVR.ContestSelection")]
    #[default]
    ContestSelection,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct ContestSelection {
    #[serde(rename = "@id")]
    pub id: String,

    // #[serde(rename = "@type")]
    // pub object_type: ContestSelectionObjectType,
    /// Code used to identify the contest selection.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,
}

/// Election defines instances of the Contest and Candidate classes so that they can be later referenced in CVR classes.  Election includes an instance of Contest for each contest in the election and includes an instance of Candidate for each candidate.  This is done to utilize file sizes more efficiently; otherwise each CVR would need to define these instances separately and much duplication would occur.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum ElectionObjectType {
    #[serde(rename = "CVR.Election")]
    #[default]
    Election,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct Election {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: ElectionObjectType,

    /// Used to establish a collection of candidate definitions that will be referenced by the CVRs.  The contests in each CVR will reference the candidate definitions.
    #[serde(rename = "Candidate", skip_serializing_if = "Option::is_none")]
    pub candidate: Option<Vec<Candidate>>,

    /// Used for a code associated with the election, e.g., a precinct identifier if the election scope is a precinct.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Used for establishing a collection of contest definitions that will be referenced by the CVRs.
    #[serde(rename = "Contest")]
    pub contest: Vec<AnyContest>,

    /// Used to identify the election scope, i.e., the political geography corresponding to the election.
    #[serde(rename = "ElectionScopeId")]
    pub election_scope_id: String,

    /// A text string identifying the election.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum FileObjectType {
    #[serde(rename = "CVR.File")]
    #[default]
    File,
}

/// Used to hold the contents of a file or identify a file created by the scanning device.  The file generally would contain an image of the scanned ballot or an image of a write-in entered by a voter onto the scanned ballot.  SubClass Image is used if the file contains an image.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct File {
    #[serde(rename = "@type")]
    pub object_type: FileObjectType,

    #[serde(rename = "Data")]
    pub data: String,

    /// Contains the name of the file or an identifier of the file.
    #[serde(rename = "FileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,

    /// The mime type of the file, e.g., image/jpeg.
    #[serde(rename = "MimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Used for identifying a geographical unit for various purposes, including:
///
/// The reporting unit of the report generation device, e.g., a precinct location of a scanner that generates the collection of CVRs,
///
/// The geographical scope of the election, or the unit of geography associated with an individual CVR.
///
/// CastVoteRecordReport includes instances of GpUnit as needed. Election references GpUnit as ElectionScope, for the geographical scope of the election.  CVR CastVoteRecordReport includes instances of GpUnit as needed. Election references GpUnit as ElectionScope, for the geographical scope of the election.  CVR references GpUnit as BallotStyleUnit to link a CVR to the smallest political subdivision that uses the same ballot style as was used for the voter's ballot.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum GpUnitObjectType {
    #[serde(rename = "CVR.GpUnit")]
    #[default]
    GpUnit,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct GpUnit {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: GpUnitObjectType,

    /// A code associated with the geographical unit.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Name of the geographical unit.
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Used when Type is 'other' to include a user-defined type.
    #[serde(rename = "OtherType", skip_serializing_if = "Option::is_none")]
    pub other_type: Option<String>,

    /// The collection of cast vote records associated with the reporting unit and the reporting device.
    #[serde(rename = "ReportingDeviceIds", skip_serializing_if = "Option::is_none")]
    pub reporting_device_ids: Option<Vec<String>>,

    /// Contains the type of geographical unit, e.g., precinct, split-precinct, vote center, using values from the ReportingUnitType enumeration.  If no values apply, use 'other' and include a user-defined type in OtherType.
    #[serde(rename = "Type")]
    pub gp_unit_type: ReportingUnitType,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum HashObjectType {
    #[serde(rename = "CVR.Hash")]
    #[default]
    Hash,
}

/// Hash is used to specify a hash associated with a file such as an image file of a scanned ballot.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct Hash {
    #[serde(rename = "@type")]
    pub object_type: HashObjectType,

    /// If Type is 'other', the type of the hash.
    #[serde(rename = "OtherType", skip_serializing_if = "Option::is_none")]
    pub other_type: Option<String>,

    /// The type of the hash, from the HashType enumeration.
    #[serde(rename = "Type")]
    pub hash_type: HashType,

    /// The hash value, encoded as a string.
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum ImageObjectType {
    #[serde(rename = "CVR.Image")]
    #[default]
    Image,
}

/// Used by File for a file containing an image, e.g., an image of a write-in on a paper ballot.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct Image {
    #[serde(rename = "@type")]
    pub object_type: ImageObjectType,

    #[serde(rename = "Data")]
    pub data: String,

    /// Contains the name of the file or an identifier of the file.
    #[serde(rename = "FileName", skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,

    /// The mime type of the file, e.g., image/jpeg.
    #[serde(rename = "MimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum ImageDataObjectType {
    #[serde(rename = "CVR.ImageData")]
    #[default]
    ImageData,
}

/// ImageData is used to specify an image file such as for a write-in or the entire ballot.  It works with several other classes, as follows:
///
/// File with SubClass Image – to contain either a filename for an external file or the file contents, and
///
/// Hash – to contain cryptographic hash function data for the file.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct ImageData {
    #[serde(rename = "@type")]
    pub object_type: ImageDataObjectType,

    /// A hash value for the image data, used for verification comparisons against subsequent copies of the image.
    #[serde(rename = "Hash", skip_serializing_if = "Option::is_none")]
    pub hash: Option<Hash>,

    /// The image of an individual ballot sheet created by the scanner, could possibly include both sides of a two-sided ballot sheet depending on the scanner's configuration.
    #[serde(rename = "Image", skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,

    /// A pointer to the location of the image file.
    #[serde(rename = "Location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Party is used for describing information about a political party associated with the voter's ballot.  CVR includes instances of Party as needed, e.g., for a CVR corresponding to a ballot in a partisan primary, and CandidateContest references Party as needed to link a candidate to their political party.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum PartyObjectType {
    #[serde(rename = "CVR.Party")]
    #[default]
    Party,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct Party {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: PartyObjectType,

    /// Short name for the party, e.g., "DEM".
    #[serde(rename = "Abbreviation", skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,

    /// A code associated with the party.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Official full name of the party, e.g., "Republican".
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// PartyContest is a subclass of Contest and is used to identify the type of contest as involving a straight party selection.  It inherits attributes from Contest.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum PartyContestObjectType {
    #[serde(rename = "CVR.PartyContest")]
    #[default]
    PartyContest,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct PartyContest {
    #[serde(rename = "@id")]
    pub id: String,

    // #[serde(rename = "@type")]
    // pub object_type: PartyContestObjectType,
    /// An abbreviation associated with the contest.
    #[serde(rename = "Abbreviation", skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,

    /// A code or identifier used for this contest.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Identifies the contest selections in the contest.
    #[serde(rename = "ContestSelection")]
    pub contest_selection: Vec<AnyContestSelection>,

    /// Title or name of the contest, e.g., "Governor" or "Question on Legalization of Gambling".
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// If VoteVariation is 'other', the vote variation for this contest.
    #[serde(rename = "OtherVoteVariation", skip_serializing_if = "Option::is_none")]
    pub other_vote_variation: Option<String>,

    /// The vote variation for this contest, from the VoteVariation enumeration.
    #[serde(rename = "VoteVariation", skip_serializing_if = "Option::is_none")]
    pub vote_variation: Option<VoteVariation>,
}

/// PartySelection is a subclass of ContestSelection and is used typically for a contest selection in a straight-party contest.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum PartySelectionObjectType {
    #[serde(rename = "CVR.PartySelection")]
    #[default]
    PartySelection,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct PartySelection {
    #[serde(rename = "@id")]
    pub id: String,

    // #[serde(rename = "@type")]
    // pub object_type: PartySelectionObjectType,
    /// Code used to identify the contest selection.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// The party associated with the contest selection.
    #[serde(rename = "PartyIds")]
    pub party_ids: Vec<String>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum ReportingDeviceObjectType {
    #[serde(rename = "CVR.ReportingDevice")]
    #[default]
    ReportingDeviceType,
}

/// ReportingDevice is used to specify a voting device as the "political geography" at hand.  CastVoteRecordReport refers to it as ReportGeneratingDevice and uses it to specify the device that created the CVR report. CVR refers to it as CreatingDevice to specify the device that created the CVRs.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct ReportingDevice {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: ReportingDeviceObjectType,

    /// The application associated with the reporting device.
    #[serde(rename = "Application", skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,

    /// A code associated with the reporting device.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Manufacturer of the reporting device.
    #[serde(rename = "Manufacturer", skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,

    /// The type of metric being used to determine quality.  The type must be specific enough that the attached value can be accurately verified later, e.g., 'Acme Mark Density' may be a sufficiently specific type.
    #[serde(rename = "MarkMetricType", skip_serializing_if = "Option::is_none")]
    pub mark_metric_type: Option<String>,

    /// Manufacturer's model of the reporting device.
    #[serde(rename = "Model", skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Additional explanatory notes as applicable.
    #[serde(rename = "Notes", skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<String>>,

    /// Serial number or other identification that can uniquely identify the reporting device.
    #[serde(rename = "SerialNumber", skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
}

/// RetentionContest is a subclass of BallotMeasureContest and is used to identify the type of contest as involving a retention, such as for a judicial retention.  While it is similar to BallotMeasureContest, it contains a link to Candidate that BallotMeasureContest does not.  RetentionContest inherits attributes from Contest.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum RetentionContestObjectType {
    #[serde(rename = "CVR.RetentionContest")]
    #[default]
    RetentionContest,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize)]
pub struct RetentionContest {
    #[serde(rename = "@id")]
    pub id: String,

    #[serde(rename = "@type")]
    pub object_type: RetentionContestObjectType,

    /// An abbreviation associated with the contest.
    #[serde(rename = "Abbreviation", skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,

    /// Identifies the candidate in the retention contest.
    #[serde(rename = "CandidateId", skip_serializing_if = "Option::is_none")]
    pub candidate_id: Option<String>,

    /// A code or identifier used for this contest.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// Identifies the contest selections in the contest.
    #[serde(rename = "ContestSelection")]
    pub contest_selection: Vec<AnyContestSelection>,

    /// Title or name of the contest, e.g., "Governor" or "Question on Legalization of Gambling".
    #[serde(rename = "Name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// If VoteVariation is 'other', the vote variation for this contest.
    #[serde(rename = "OtherVoteVariation", skip_serializing_if = "Option::is_none")]
    pub other_vote_variation: Option<String>,

    /// The vote variation for this contest, from the VoteVariation enumeration.
    #[serde(rename = "VoteVariation", skip_serializing_if = "Option::is_none")]
    pub vote_variation: Option<VoteVariation>,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub enum SelectionPositionObjectType {
    #[serde(rename = "CVR.SelectionPosition")]
    #[default]
    SelectionPosition,
}

/// CVRContestSelection includes SelectionPosition to specify a voter's indication/mark in a contest option, and thus, a potential vote. The number of potential SelectionPositions that could be included by CVRContestSelection is the same as the number of ovals next to a particular option. There will be usually 1 instance of SelectionPosition for plurality voting, but there could be multiple instances for RCV, approval, cumulative, or other vote variations in which a voter can select multiple options per candidate.   MarkMetricValue specifies the measurement of a mark on a paper ballot. The measurement is assigned by the scanner for measurements of mark density or quality and would be used by the scanner to indicate whether the mark is a valid voter mark representing a vote or is marginal.SelectionPosition contains additional information about the mark to specify whether the indication/mark is allocable, as well as information needed for certain voting methods.SelectionPosition includes CVRWriteIn, whose attributes are used to include information about the write-in including the text of the write-in or an image of the write-in.
#[derive(Clone, Eq, Hash, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct SelectionPosition {
    #[serde(rename = "@type")]
    pub object_type: SelectionPositionObjectType,

    /// Used to store information regarding a write-in vote.
    #[serde(rename = "CVRWriteIn", skip_serializing_if = "Option::is_none")]
    pub cvr_write_in: Option<CVRWriteIn>,

    /// Code used to identify the contest selection position.
    #[serde(rename = "Code", skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Code>>,

    /// The proper fractional number of votes represented by the position.
    #[serde(rename = "FractionalVotes", skip_serializing_if = "Option::is_none")]
    pub fractional_votes: Option<FractionalNumber>,

    /// Whether there is a selection indication present.
    #[serde(rename = "HasIndication")]
    pub has_indication: IndicationStatus,

    /// Whether this indication should be allocated to the contest option's accumulator.
    #[serde(rename = "IsAllocable", skip_serializing_if = "Option::is_none")]
    pub is_allocable: Option<AllocationStatus>,

    /// Whether or not the indication was generated, rather than directly made by the voter.
    #[serde(rename = "IsGenerated", skip_serializing_if = "Option::is_none")]
    pub is_generated: Option<bool>,

    /// The value of the mark metric, represented as a string.
    #[serde(rename = "MarkMetricValue", skip_serializing_if = "Option::is_none")]
    pub mark_metric_value: Option<Vec<String>>,

    /// The number of votes represented by the position, usually 1 but may be more depending on the voting method.
    #[serde(rename = "NumberVotes")]
    pub number_votes: i64,

    /// Used when Status is 'other' to include a user-defined status.
    #[serde(rename = "OtherStatus", skip_serializing_if = "Option::is_none")]
    pub other_status: Option<String>,

    /// The ordinal position of the selection position within the contest option.
    #[serde(rename = "Position", skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,

    /// For the RCV voting variation, the rank chosen by the voter, for when a position can represent a ranking.
    #[serde(rename = "Rank", skip_serializing_if = "Option::is_none")]
    pub rank: Option<i64>,

    /// Status of the position, e.g., &quot;generated-rules&quot; for generated by the machine, from the PositionStatus enumeration.  If no values apply, use 'other' and include a user-defined status in OtherStatus.
    #[serde(rename = "Status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<PositionStatus>>,
}
