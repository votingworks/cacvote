use base64_serde::base64_serde_type;
use serde::Deserialize;
use serde::Serialize;

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
        if value.is_empty() {
            Err("Jurisdiction code cannot be empty".to_owned())
        } else {
            Ok(Self(value))
        }
    }
}
