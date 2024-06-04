use std::fmt;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::str::FromStr;

use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tlv_derive::{Decode, Encode};
use uuid::Uuid;

use crate::election::BallotStyleId;
use crate::election::ElectionDefinition;
use crate::election::ElectionHash;
use crate::election::PrecinctId;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(transparent)]
pub struct JurisdictionCode(String);

impl JurisdictionCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for JurisdictionCode {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<&str> for JurisdictionCode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err("Jurisdiction code cannot be empty".to_owned())
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

impl FromStr for JurisdictionCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl fmt::Display for JurisdictionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "sqlx")]
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for JurisdictionCode
where
    sqlx::types::Json<Self>: sqlx::Decode<'r, sqlx::Postgres>,
{
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(value.as_str()?.try_into()?)
    }
}

#[cfg(feature = "sqlx")]
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for JurisdictionCode
where
    Vec<u8>: sqlx::Encode<'q, sqlx::Postgres>,
{
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        self.0.encode_by_ref(buf)
    }
}

#[cfg(feature = "sqlx")]
impl sqlx::Type<sqlx::Postgres> for JurisdictionCode {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("varchar")
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedObject {
    pub id: Uuid,

    /// The jurisdiction code of the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub election_id: Option<Uuid>,

    /// Data to be signed. Must be JSON decodable as [`Payload`][crate::cacvote::Payload].
    #[serde(with = "Base64Standard")]
    pub payload: Vec<u8>,

    /// A stack of PEM-encoded X.509 certificates.
    #[serde(with = "Base64Standard")]
    pub certificates: Vec<u8>,

    /// The signature of the payload.
    #[serde(with = "Base64Standard")]
    pub signature: Vec<u8>,
}

impl SignedObject {
    #[cfg(feature = "openssl")]
    pub fn from_payload(
        payload: &Payload,
        certificates: Vec<openssl::x509::X509>,
        private_key: &openssl::pkey::PKeyRef<openssl::pkey::Private>,
    ) -> color_eyre::Result<Self> {
        let election_id = payload.election_id();
        let mut signer =
            openssl::sign::Signer::new(openssl::hash::MessageDigest::sha256(), private_key)?;
        let payload = serde_json::to_vec(payload)?;
        signer.update(&payload)?;
        let signature = signer.sign_to_vec()?;

        let certificates = certificates
            .iter()
            .map(|cert| cert.to_pem())
            .collect::<Result<Vec<_>, _>>()?
            .concat();

        Ok(Self {
            id: Uuid::new_v4(),
            election_id,
            payload,
            certificates,
            signature,
        })
    }

    pub fn try_to_inner(&self) -> Result<Payload, serde_json::Error> {
        serde_json::from_slice(&self.payload)
    }

    #[cfg(feature = "openssl")]
    pub fn to_x509(&self) -> Result<Vec<openssl::x509::X509>, openssl::error::ErrorStack> {
        openssl::x509::X509::stack_from_pem(&self.certificates)
    }

    #[cfg(feature = "openssl")]
    pub fn verify(&self) -> Result<bool, openssl::error::ErrorStack> {
        let public_key = match self.to_x509()?.first() {
            Some(x509) => x509.public_key()?,
            None => return Ok(false),
        };

        let mut verifier =
            openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), &public_key)?;
        verifier.update(&self.payload)?;
        verifier.verify(&self.signature)
    }

    #[must_use]
    pub fn jurisdiction_code(&self) -> Option<JurisdictionCode> {
        let jurisdiction_code = self
            .try_to_inner()
            .ok()
            .map(|payload| payload.jurisdiction_code());

        #[cfg(feature = "openssl")]
        {
            jurisdiction_code.or_else(|| self.jurisdiction_code_from_certificates())
        }

        #[cfg(not(feature = "openssl"))]
        {
            jurisdiction_code
        }
    }

    #[cfg(feature = "openssl")]
    #[must_use]
    pub fn jurisdiction_code_from_certificates(&self) -> Option<JurisdictionCode> {
        /// Format: {state-2-letter-abbreviation}.{county-or-town} (e.g. ms.warren or ca.los-angeles)
        const VX_CUSTOM_CERT_FIELD_JURISDICTION: &str = "1.3.6.1.4.1.59817.2";

        self.to_x509()
            .ok()?
            .first()?
            .subject_name()
            .entries()
            .find(|entry| entry.object().to_string() == VX_CUSTOM_CERT_FIELD_JURISDICTION)?
            .data()
            .as_utf8()
            .ok()?
            .to_string()
            .try_into()
            .ok()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase", tag = "objectType")]
pub enum Payload {
    RegistrationRequest(RegistrationRequest),
    Registration(Registration),
    Election(Election),
    CastBallot(CastBallot),
    EncryptedElectionTally(EncryptedElectionTally),
    DecryptedElectionTally(DecryptedElectionTally),
    ShuffledEncryptedCastBallots(ShuffledEncryptedCastBallots),
}

impl Payload {
    pub fn object_type(&self) -> &'static str {
        match self {
            Self::RegistrationRequest(_) => Self::registration_request_object_type(),
            Self::Registration(_) => Self::registration_object_type(),
            Self::Election(_) => Self::election_object_type(),
            Self::CastBallot(_) => Self::cast_ballot_object_type(),
            Self::EncryptedElectionTally(_) => Self::encrypted_election_tally_object_type(),
            Self::DecryptedElectionTally(_) => Self::decrypted_election_tally_object_type(),
            Self::ShuffledEncryptedCastBallots(_) => {
                Self::shuffled_encrypted_cast_ballots_object_type()
            }
        }
    }

    pub fn election_id(&self) -> Option<Uuid> {
        match self {
            Self::RegistrationRequest(_) => None,
            Self::Registration(r) => Some(r.election_object_id),
            Self::Election(_) => None,
            Self::CastBallot(cb) => Some(cb.election_object_id),
            Self::EncryptedElectionTally(tally) => Some(tally.election_object_id),
            Self::DecryptedElectionTally(tally) => Some(tally.election_object_id),
            Self::ShuffledEncryptedCastBallots(ballots) => Some(ballots.election_object_id),
        }
    }

    pub fn registration_request_object_type() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Payload` enum.
        "RegistrationRequest"
    }

    pub fn registration_object_type() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Payload` enum.
        "Registration"
    }

    pub fn election_object_type() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Payload` enum.
        "Election"
    }

    pub fn cast_ballot_object_type() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Payload` enum.
        "CastBallot"
    }

    pub fn encrypted_election_tally_object_type() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Payload` enum.
        "EncryptedElectionTally"
    }

    pub fn decrypted_election_tally_object_type() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Payload` enum.
        "DecryptedElectionTally"
    }

    pub fn shuffled_encrypted_cast_ballots_object_type() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Payload` enum.
        "ShuffledEncryptedCastBallots"
    }
}

impl JurisdictionScoped for Payload {
    fn jurisdiction_code(&self) -> JurisdictionCode {
        match self {
            Self::RegistrationRequest(request) => request.jurisdiction_code(),
            Self::Registration(registration) => registration.jurisdiction_code(),
            Self::Election(election) => election.jurisdiction_code(),
            Self::CastBallot(cast_ballot) => cast_ballot.jurisdiction_code(),
            Self::EncryptedElectionTally(tally) => tally.jurisdiction_code(),
            Self::DecryptedElectionTally(tally) => tally.jurisdiction_code(),
            Self::ShuffledEncryptedCastBallots(ballots) => ballots.jurisdiction_code(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub id: Uuid,
    pub object_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub election_id: Option<Uuid>,
    pub jurisdiction_code: JurisdictionCode,
    pub object_type: String,
    pub action: JournalEntryAction,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalEntryAction {
    Create,
    Delete,
    Unknown(String),
}

impl JournalEntryAction {
    pub fn as_str(&self) -> &str {
        match self {
            JournalEntryAction::Create => "create",
            JournalEntryAction::Delete => "delete",
            JournalEntryAction::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for JournalEntryAction {
    fn from(s: &str) -> Self {
        match s {
            "create" => JournalEntryAction::Create,
            "delete" => JournalEntryAction::Delete,
            _ => JournalEntryAction::Unknown(s.to_owned()),
        }
    }
}

impl From<&JournalEntryAction> for String {
    fn from(action: &JournalEntryAction) -> Self {
        action.as_str().to_owned()
    }
}

impl From<JournalEntryAction> for String {
    fn from(action: JournalEntryAction) -> Self {
        String::from(&action)
    }
}

impl From<String> for JournalEntryAction {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl<'de> Deserialize<'de> for JournalEntryAction {
    fn deserialize<D>(deserializer: D) -> Result<JournalEntryAction, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(String::deserialize(deserializer)?.into())
    }
}

impl Serialize for JournalEntryAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "sqlx")]
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for JournalEntryAction
where
    sqlx::types::Json<Self>: sqlx::Decode<'r, sqlx::Postgres>,
{
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(value.as_str()?.into())
    }
}

#[cfg(feature = "sqlx")]
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for JournalEntryAction
where
    Vec<u8>: sqlx::Encode<'q, sqlx::Postgres>,
{
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        String::from(self).encode_by_ref(buf)
    }
}

#[cfg(feature = "sqlx")]
impl sqlx::Type<sqlx::Postgres> for JournalEntryAction {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("varchar")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum SmartcardStatus {
    #[default]
    NoReader,
    NoCard,
    Card,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum VerificationStatus {
    #[serde(rename_all = "camelCase")]
    Success {
        common_access_card_id: String,
        display_name: String,
    },
    #[serde(rename_all = "camelCase")]
    Failure,
    #[serde(rename_all = "camelCase")]
    Error { message: String },
    #[default]
    #[serde(rename_all = "camelCase")]
    Unknown,
}

trait JurisdictionScoped {
    fn jurisdiction_code(&self) -> JurisdictionCode;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequest {
    pub common_access_card_id: String,
    pub jurisdiction_code: JurisdictionCode,
    pub given_name: String,
    pub family_name: String,
}

impl JurisdictionScoped for RegistrationRequest {
    fn jurisdiction_code(&self) -> JurisdictionCode {
        self.jurisdiction_code.clone()
    }
}

impl RegistrationRequest {
    pub fn common_access_card_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `RegistrationRequest` struct.
        "commonAccessCardId"
    }

    pub fn jurisdiction_code_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `RegistrationRequest` struct.
        "jurisdictionCode"
    }

    pub fn given_name_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `RegistrationRequest` struct.
        "givenName"
    }

    pub fn family_name_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `RegistrationRequest` struct.
        "familyName"
    }

    pub fn display_name(&self) -> String {
        format!("{} {}", self.given_name, self.family_name)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub common_access_card_id: String,
    pub jurisdiction_code: JurisdictionCode,
    pub registration_request_object_id: Uuid,
    pub election_object_id: Uuid,
    pub ballot_style_id: BallotStyleId,
    pub precinct_id: PrecinctId,
}

impl JurisdictionScoped for Registration {
    fn jurisdiction_code(&self) -> JurisdictionCode {
        self.jurisdiction_code.clone()
    }
}

impl Registration {
    pub fn common_access_card_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Registration` struct.
        "commonAccessCardId"
    }

    pub fn jurisdiction_code_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Registration` struct
        "jurisdictionCode"
    }

    pub fn registration_request_object_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Registration` struct
        "registrationRequestObjectId"
    }

    pub fn election_object_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Registration` struct
        "electionObjectId"
    }

    pub fn ballot_style_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Registration` struct
        "ballotStyleId"
    }

    pub fn precinct_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `Registration` struct
        "precinctId"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateElectionRequest {
    pub jurisdiction_code: JurisdictionCode,
    pub election_definition: ElectionDefinition,
    pub mailing_address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Election {
    pub jurisdiction_code: JurisdictionCode,
    pub election_definition: ElectionDefinition,
    pub mailing_address: String,

    #[serde(with = "Base64Standard")]
    pub electionguard_election_metadata_blob: Vec<u8>,
}

impl JurisdictionScoped for Election {
    fn jurisdiction_code(&self) -> JurisdictionCode {
        self.jurisdiction_code.clone()
    }
}

impl Deref for Election {
    type Target = ElectionDefinition;

    fn deref(&self) -> &Self::Target {
        &self.election_definition
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CastBallot {
    pub common_access_card_id: String,
    pub jurisdiction_code: JurisdictionCode,
    pub registration_request_object_id: Uuid,
    pub registration_object_id: Uuid,
    pub election_object_id: Uuid,

    #[serde(with = "Base64Standard")]
    pub electionguard_encrypted_ballot: Vec<u8>,
}

impl CastBallot {
    pub fn registration_request_object_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `CastBallot` struct.
        "registrationRequestObjectId"
    }

    pub fn registration_object_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `CastBallot` struct.
        "registrationObjectId"
    }

    pub fn election_object_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `CastBallot` struct.
        "electionObjectId"
    }
}

impl JurisdictionScoped for CastBallot {
    fn jurisdiction_code(&self) -> JurisdictionCode {
        self.jurisdiction_code.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CastBallotPresenter {
    cast_ballot: CastBallot,
    registration_request: RegistrationRequest,
    registration: Registration,
    registration_id: Uuid,
    verification_status: VerificationStatus,
    #[serde(with = "time::serde::iso8601")]
    created_at: OffsetDateTime,
}

impl CastBallotPresenter {
    pub const fn new(
        cast_ballot: CastBallot,
        registration_request: RegistrationRequest,
        registration: Registration,
        registration_id: Uuid,
        verification_status: VerificationStatus,
        created_at: OffsetDateTime,
    ) -> Self {
        Self {
            cast_ballot,
            registration_request,
            registration,
            registration_id,
            verification_status,
            created_at,
        }
    }

    pub fn registration(&self) -> &Registration {
        &self.registration
    }

    pub fn registration_id(&self) -> &Uuid {
        &self.registration_id
    }

    pub fn registration_request(&self) -> &RegistrationRequest {
        &self.registration_request
    }

    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }

    pub fn verification_status(&self) -> &VerificationStatus {
        &self.verification_status
    }
}

impl Deref for CastBallotPresenter {
    type Target = CastBallot;

    fn deref(&self) -> &Self::Target {
        &self.cast_ballot
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedElectionTally {
    pub jurisdiction_code: JurisdictionCode,
    pub election_object_id: Uuid,
    #[serde(with = "Base64Standard")]
    pub electionguard_encrypted_tally: Vec<u8>,
}

impl EncryptedElectionTally {
    pub fn election_object_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `EncryptedElectionTally` struct.
        "electionObjectId"
    }
}

impl JurisdictionScoped for EncryptedElectionTally {
    fn jurisdiction_code(&self) -> JurisdictionCode {
        self.jurisdiction_code.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedElectionTallyPresenter {
    pub encrypted_election_tally: EncryptedElectionTally,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(
        with = "time::serde::iso8601::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub synced_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptedElectionTally {
    pub jurisdiction_code: JurisdictionCode,
    pub election_object_id: Uuid,
    #[serde(with = "Base64Standard")]
    pub electionguard_decrypted_tally: Vec<u8>,
}

impl DecryptedElectionTally {
    pub fn election_object_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `DecryptedElectionTally` struct.
        "electionObjectId"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptedElectionTallyPresenter {
    pub decrypted_election_tally: DecryptedElectionTally,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(
        with = "time::serde::iso8601::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub synced_at: Option<OffsetDateTime>,
}

impl JurisdictionScoped for DecryptedElectionTally {
    fn jurisdiction_code(&self) -> JurisdictionCode {
        self.jurisdiction_code.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShuffledEncryptedCastBallots {
    pub jurisdiction_code: JurisdictionCode,
    pub election_object_id: Uuid,
    #[serde(with = "Base64Standard")]
    pub electionguard_shuffled_ballots: Vec<u8>,
}

impl JurisdictionScoped for ShuffledEncryptedCastBallots {
    fn jurisdiction_code(&self) -> JurisdictionCode {
        self.jurisdiction_code.clone()
    }
}

impl ShuffledEncryptedCastBallots {
    pub fn election_object_id_field_name() -> &'static str {
        // This must match the naming rules of the `serde` attribute in the
        // `ShuffledEncryptedCastBallots` struct.
        "electionObjectId"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShuffledEncryptedCastBallotsPresenter {
    #[serde(flatten)]
    pub shuffled_encrypted_cast_ballots: ShuffledEncryptedCastBallots,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(
        with = "time::serde::iso8601::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub synced_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SessionData {
    #[serde(rename_all = "camelCase")]
    Authenticated {
        jurisdiction_code: JurisdictionCode,
        elections: Vec<ElectionPresenter>,
        pending_registration_requests: Vec<RegistrationRequestPresenter>,
        registrations: Vec<RegistrationPresenter>,
        cast_ballots: Vec<CastBallotPresenter>,
    },
    #[serde(rename_all = "camelCase")]
    Unauthenticated { has_smartcard: bool },
}

impl Default for SessionData {
    fn default() -> Self {
        Self::Unauthenticated {
            has_smartcard: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRegistrationRequest {
    pub election_id: Uuid,
    pub registration_request_id: Uuid,
    pub ballot_style_id: BallotStyleId,
    pub precinct_id: PrecinctId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElectionPresenter {
    pub id: Uuid,
    election: Election,
    #[serde(skip_serializing_if = "Option::is_none")]
    encrypted_tally: Option<EncryptedElectionTallyPresenter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decrypted_tally: Option<DecryptedElectionTallyPresenter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shuffled_encrypted_cast_ballots: Option<ShuffledEncryptedCastBallotsPresenter>,
}

impl ElectionPresenter {
    pub fn new(
        id: Uuid,
        election: Election,
        encrypted_tally: Option<EncryptedElectionTallyPresenter>,
        decrypted_tally: Option<DecryptedElectionTallyPresenter>,
        shuffled_encrypted_cast_ballots: Option<ShuffledEncryptedCastBallotsPresenter>,
    ) -> Self {
        Self {
            id,
            election,
            encrypted_tally,
            decrypted_tally,
            shuffled_encrypted_cast_ballots,
        }
    }
}

impl Deref for ElectionPresenter {
    type Target = Election;

    fn deref(&self) -> &Self::Target {
        &self.election
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationRequestPresenter {
    pub id: Uuid,
    display_name: String,
    registration_request: RegistrationRequest,
    #[serde(with = "time::serde::iso8601")]
    created_at: OffsetDateTime,
}

impl Deref for RegistrationRequestPresenter {
    type Target = RegistrationRequest;

    fn deref(&self) -> &Self::Target {
        &self.registration_request
    }
}

impl RegistrationRequestPresenter {
    pub fn new(
        id: Uuid,
        display_name: String,
        registration_request: RegistrationRequest,
        created_at: OffsetDateTime,
    ) -> Self {
        Self {
            id,
            display_name,
            registration_request,
            created_at,
        }
    }

    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationPresenter {
    pub id: Uuid,
    display_name: String,
    election_title: String,
    election_hash: ElectionHash,
    registration: Registration,
    #[serde(with = "time::serde::iso8601")]
    created_at: OffsetDateTime,
    is_synced: bool,
}

impl Deref for RegistrationPresenter {
    type Target = Registration;

    fn deref(&self) -> &Self::Target {
        &self.registration
    }
}

impl RegistrationPresenter {
    pub fn new(
        id: Uuid,
        display_name: String,
        election_title: String,
        election_hash: ElectionHash,
        registration: Registration,
        created_at: OffsetDateTime,
        is_synced: bool,
    ) -> Self {
        Self {
            id,
            display_name,
            election_title,
            election_hash,
            registration,
            created_at,
            is_synced,
        }
    }

    pub fn display_name(&self) -> String {
        self.display_name.clone()
    }

    pub fn election_title(&self) -> String {
        self.election_title.clone()
    }

    pub fn election_hash(&self) -> ElectionHash {
        self.election_hash.clone()
    }

    pub fn precinct_id(&self) -> PrecinctId {
        self.precinct_id.clone()
    }

    pub fn ballot_style_id(&self) -> BallotStyleId {
        self.ballot_style_id.clone()
    }

    pub fn is_synced(&self) -> bool {
        self.is_synced
    }

    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixEncryptedBallotsRequest {
    pub phases: NonZeroUsize,
}

/// A payload for verifying a ballot. This payload is encoded as a TLV structure.
#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BallotVerificationPayload {
    /// The machine ID of the voting machine.
    #[tlv(tag = 0x02)]
    machine_id: String,

    /// The common access card ID of the voter.
    #[tlv(tag = 0x03)]
    common_access_card_id: String,

    /// The election object ID from the database.
    #[tlv(tag = 0x04)]
    election_object_id: Uuid,

    /// The SHA-256 hash of the encrypted ballot signature.
    #[tlv(tag = 0x05)]
    #[serde(with = "Base64Standard")]
    encrypted_ballot_signature_hash: [u8; 32],
}

impl BallotVerificationPayload {
    /// Creates a new ballot verification payload.
    pub const fn new(
        machine_id: String,
        common_access_card_id: String,
        election_object_id: Uuid,
        encrypted_ballot_signature_hash: [u8; 32],
    ) -> Self {
        Self {
            machine_id,
            common_access_card_id,
            election_object_id,
            encrypted_ballot_signature_hash,
        }
    }

    /// Returns the machine ID of the voting machine.
    pub fn machine_id(&self) -> &str {
        &self.machine_id
    }

    /// Returns the common access card ID of the voter.
    pub fn common_access_card_id(&self) -> &str {
        &self.common_access_card_id
    }

    /// Returns the election object ID from the database.
    pub fn election_object_id(&self) -> Uuid {
        self.election_object_id
    }

    /// Returns the SHA-256 hash of the encrypted ballot signature.
    pub fn encrypted_ballot_signature_hash(&self) -> &[u8; 32] {
        &self.encrypted_ballot_signature_hash
    }
}

/// A buffer that has been signed.
#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize)]
pub struct SignedBuffer {
    /// The buffer that was signed.
    #[tlv(tag = 0x06)]
    #[serde(with = "Base64Standard")]
    buffer: Vec<u8>,

    /// The signature of the buffer.
    #[tlv(tag = 0x07)]
    #[serde(with = "Base64Standard")]
    signature: Vec<u8>,
}

impl SignedBuffer {
    /// Creates a new signed buffer.
    pub const fn new(buffer: Vec<u8>, signature: Vec<u8>) -> Self {
        Self { buffer, signature }
    }

    /// Returns the buffer that was signed.
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns the signature of the buffer.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Decodes the buffer into a type that implements the `tlv::Decode` trait.
    pub fn decode_buffer<D>(&self) -> tlv::Result<D>
    where
        D: tlv::Decode,
    {
        let decoded: D = tlv::from_slice(&self.buffer)?;
        Ok(decoded)
    }

    #[cfg(feature = "openssl")]
    pub fn verify(
        &self,
        public_key: &openssl::pkey::PKeyRef<openssl::pkey::Public>,
    ) -> Result<bool, openssl::error::ErrorStack> {
        let mut verifier =
            openssl::sign::Verifier::new(openssl::hash::MessageDigest::sha256(), public_key)?;
        verifier.update(&self.buffer)?;
        verifier.verify(&self.signature)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ballot_verification_payload() {
        let machine_id = "machine-id".to_owned();
        let common_access_card_id = "common-access-card-id".to_owned();
        let election_object_id = uuid::Uuid::new_v4();
        let encrypted_ballot_signature_hash = [0; 32];

        let payload = crate::cacvote::BallotVerificationPayload::new(
            machine_id.clone(),
            common_access_card_id.clone(),
            election_object_id,
            encrypted_ballot_signature_hash,
        );

        let encoded = tlv::to_vec(payload).unwrap();
        let decoded: crate::cacvote::BallotVerificationPayload = tlv::from_slice(&encoded).unwrap();

        assert_eq!(decoded.machine_id(), machine_id);
        assert_eq!(decoded.common_access_card_id(), common_access_card_id);
        assert_eq!(decoded.election_object_id(), election_object_id);
        assert_eq!(
            decoded.encrypted_ballot_signature_hash(),
            &encrypted_ballot_signature_hash
        );
    }
}
