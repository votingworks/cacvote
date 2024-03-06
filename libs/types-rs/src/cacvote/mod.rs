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
    pub action: String,
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug)]
pub enum JournalEntryAction {
    Create,
    Delete,
    Unknown(String),
}

impl<'de> Deserialize<'de> for JournalEntryAction {
    fn deserialize<D>(deserializer: D) -> Result<JournalEntryAction, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "create" => Ok(JournalEntryAction::Create),
            "delete" => Ok(JournalEntryAction::Delete),
            _ => Ok(JournalEntryAction::Unknown(s)),
        }
    }
}

impl Serialize for JournalEntryAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            JournalEntryAction::Create => serializer.serialize_str("create"),
            JournalEntryAction::Delete => serializer.serialize_str("delete"),
            JournalEntryAction::Unknown(s) => serializer.serialize_str(s),
        }
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
