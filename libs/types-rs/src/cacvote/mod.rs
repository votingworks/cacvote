use base64_serde::base64_serde_type;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

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
pub struct SignedObject {
    #[serde(with = "Base64Standard")]
    pub payload: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub certificates: Vec<u8>,
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
    #[must_use]
    pub fn verify(&self) -> Result<bool, openssl::error::ErrorStack> {
        let public_key = match self.to_x509()?.first() {
            Some(x509) => x509.public_key()?,
            None => return Ok(false),
        };
        let digest = openssl::hash::MessageDigest::sha256();
        let mut verifier = openssl::sign::Verifier::new(digest, &public_key)?;
        verifier.update(&self.payload)?;
        verifier.verify(&self.signature)
    }

    #[cfg(feature = "openssl")]
    #[must_use]
    pub fn jurisdiction_code(&self) -> Option<JurisdictionCode> {
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
pub struct Payload {
    pub object_type: String,
    #[serde(with = "Base64Standard")]
    pub data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct JournalEntry {
    pub id: Uuid,
    pub object_id: Uuid,
    pub jurisdiction: JurisdictionCode,
    pub object_type: String,
    pub action: JournalEntryAction,
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
