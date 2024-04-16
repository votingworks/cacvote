use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use base64_serde::base64_serde_type;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::cdf::cvr::Cvr;
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
}

impl Payload {
    pub fn object_type(&self) -> &'static str {
        match self {
            Self::RegistrationRequest(_) => Self::registration_request_object_type(),
            Self::Registration(_) => Self::registration_object_type(),
            Self::Election(_) => Self::election_object_type(),
            Self::CastBallot(_) => Self::cast_ballot_object_type(),
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
}

impl JurisdictionScoped for Payload {
    fn jurisdiction_code(&self) -> JurisdictionCode {
        match self {
            Self::RegistrationRequest(request) => request.jurisdiction_code(),
            Self::Registration(registration) => registration.jurisdiction_code(),
            Self::Election(election) => election.jurisdiction_code(),
            Self::CastBallot(cast_ballot) => cast_ballot.jurisdiction_code(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub id: Uuid,
    pub object_id: Uuid,
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
pub enum VerificationStatus {
    Success {
        common_access_card_id: String,
        display_name: String,
    },
    Failure,
    Error(String),
    #[default]
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
pub struct Election {
    pub jurisdiction_code: JurisdictionCode,
    pub election_definition: ElectionDefinition,
    pub mailing_address: String,
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
    pub cvr: Cvr,
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

impl Deref for CastBallot {
    type Target = Cvr;

    fn deref(&self) -> &Self::Target {
        &self.cvr
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CastBallotPresenter {
    cast_ballot: CastBallot,
    registration_request: RegistrationRequest,
    registration: Registration,
    verification_status: VerificationStatus,
    #[serde(with = "time::serde::iso8601")]
    created_at: OffsetDateTime,
}

impl CastBallotPresenter {
    pub const fn new(
        cast_ballot: CastBallot,
        registration_request: RegistrationRequest,
        registration: Registration,
        verification_status: VerificationStatus,
        created_at: OffsetDateTime,
    ) -> Self {
        Self {
            cast_ballot,
            registration_request,
            registration,
            verification_status,
            created_at,
        }
    }

    pub fn registration(&self) -> &Registration {
        &self.registration
    }

    pub fn registration_request(&self) -> &RegistrationRequest {
        &self.registration_request
    }

    pub fn cvr(&self) -> &Cvr {
        &self.cvr
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
pub enum SessionData {
    Authenticated {
        jurisdiction_code: JurisdictionCode,
        elections: Vec<ElectionPresenter>,
        pending_registration_requests: Vec<RegistrationRequestPresenter>,
        registrations: Vec<RegistrationPresenter>,
        cast_ballots: Vec<CastBallotPresenter>,
    },
    Unauthenticated {
        has_smartcard: bool,
    },
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
pub struct CreateRegistrationData {
    pub election_id: Uuid,
    pub registration_request_id: Uuid,
    pub ballot_style_id: BallotStyleId,
    pub precinct_id: PrecinctId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateElectionData {
    pub election_data: String,
    pub return_address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElectionPresenter {
    pub id: Uuid,
    election: Election,
}

impl ElectionPresenter {
    pub fn new(id: Uuid, election: Election) -> Self {
        Self { id, election }
    }
}

impl Deref for ElectionPresenter {
    type Target = Election;

    fn deref(&self) -> &Self::Target {
        &self.election
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegistrationRequestPresenter {
    pub id: Uuid,
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
        registration_request: RegistrationRequest,
        created_at: OffsetDateTime,
    ) -> Self {
        Self {
            id,
            registration_request,
            created_at,
        }
    }

    pub fn display_name(&self) -> String {
        format!(
            "{} {}",
            self.registration_request.given_name, self.registration_request.family_name
        )
    }

    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
