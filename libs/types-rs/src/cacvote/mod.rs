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
    pub certificate: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub signature: Vec<u8>,
}

impl SignedObject {
    pub fn try_to_inner(&self) -> Result<Payload, serde_json::Error> {
        serde_json::from_slice(&self.payload)
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
