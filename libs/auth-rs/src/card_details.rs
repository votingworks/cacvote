use openssl::x509::X509;
use serde::{Deserialize, Serialize};
use types_rs::{
    auth::{ElectionManagerUser, PollWorkerUser, SystemAdministratorUser, User},
    cacvote::JurisdictionCode,
};

use crate::certs::{
    VX_CUSTOM_CERT_FIELD_CARD_TYPE, VX_CUSTOM_CERT_FIELD_ELECTION_HASH,
    VX_CUSTOM_CERT_FIELD_JURISDICTION,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardDetails {
    SystemAdministratorCard(SystemAdministratorCardDetails),
    ElectionManagerCard(ElectionManagerCardDetails),
    PollWorkerCard(PollWorkerCardDetails),
}

impl CardDetails {
    pub fn user(&self) -> User {
        match self {
            Self::SystemAdministratorCard(details) => {
                User::SystemAdministrator(details.user.clone())
            }
            Self::ElectionManagerCard(details) => User::ElectionManager(details.user.clone()),
            Self::PollWorkerCard(details) => User::PollWorker(details.user.clone()),
        }
    }

    pub fn jurisdiction_code(&self) -> JurisdictionCode {
        match self {
            Self::SystemAdministratorCard(details) => details.user.jurisdiction.clone(),
            Self::ElectionManagerCard(details) => details.user.jurisdiction.clone(),
            Self::PollWorkerCard(details) => details.user.jurisdiction.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PinInfo {
    NoPin,
    HasPin { num_incorrect_pin_attempts: u8 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CardDetailsWithAuthInfo {
    pub card_details: CardDetails,

    /// The certificate for the card, issued by VotingWorks.
    pub card_vx_cert: X509,

    /// The certificate for the card, issued by VxAdmin.
    pub card_vx_admin_cert: X509,

    /// The certificate authority certificate for the VxAdmin that programmed
    /// the card.
    pub vx_admin_cert_authority_cert: X509,

    /// Information about the card's PIN.
    pub pin_info: PinInfo,
}

impl CardDetailsWithAuthInfo {
    pub const fn new(
        card_details: CardDetails,
        card_vx_cert: X509,
        card_vx_admin_cert: X509,
        vx_admin_cert_authority_cert: X509,
        pin_info: PinInfo,
    ) -> Self {
        Self {
            card_details,
            card_vx_cert,
            card_vx_admin_cert,
            vx_admin_cert_authority_cert,
            pin_info,
        }
    }
}

pub fn extract_field_value(value: &X509, field_name: &str) -> Result<Option<String>, ParseError> {
    let field = value
        .subject_name()
        .entries()
        .find(|entry| entry.object().to_string() == field_name);
    Ok(Some(match field {
        Some(field) => field.data().as_utf8()?.to_string(),
        None => return Ok(None),
    }))
}

impl TryFrom<X509> for CardDetails {
    type Error = ParseError;

    fn try_from(value: X509) -> Result<Self, Self::Error> {
        let card_type = extract_field_value(&value, VX_CUSTOM_CERT_FIELD_CARD_TYPE)?
            .ok_or(ParseError::MissingCardTypeField)?;

        let jurisdiction = extract_field_value(&value, VX_CUSTOM_CERT_FIELD_JURISDICTION)?
            .ok_or(ParseError::MissingJurisdictionField)?
            .try_into()
            .map_err(ParseError::InvalidJurisdictionField)?;

        match card_type.as_str() {
            "system-administrator" => {
                let user = SystemAdministratorUser::new(jurisdiction);
                let num_incorrect_pin_attempts = None;
                Ok(Self::SystemAdministratorCard(
                    SystemAdministratorCardDetails {
                        user,
                        num_incorrect_pin_attempts,
                    },
                ))
            }
            "election-manager" => {
                let election_hash =
                    extract_field_value(&value, VX_CUSTOM_CERT_FIELD_ELECTION_HASH)?
                        .ok_or(ParseError::MissingElectionHashField)?;
                let user = ElectionManagerUser::new(jurisdiction, election_hash);
                let num_incorrect_pin_attempts = None;
                Ok(Self::ElectionManagerCard(ElectionManagerCardDetails {
                    user,
                    num_incorrect_pin_attempts,
                }))
            }
            "poll-worker" => {
                let election_hash =
                    extract_field_value(&value, VX_CUSTOM_CERT_FIELD_ELECTION_HASH)?
                        .ok_or(ParseError::MissingElectionHashField)?;
                let user = PollWorkerUser::new(jurisdiction, election_hash);
                let num_incorrect_pin_attempts = None;
                Ok(Self::PollWorkerCard(PollWorkerCardDetails {
                    user,
                    num_incorrect_pin_attempts,
                    has_pin: false,
                }))
            }
            "poll-worker-with-pin" => {
                let election_hash =
                    extract_field_value(&value, VX_CUSTOM_CERT_FIELD_ELECTION_HASH)?
                        .ok_or(ParseError::MissingElectionHashField)?;
                let user = PollWorkerUser::new(jurisdiction, election_hash);
                Ok(Self::PollWorkerCard(PollWorkerCardDetails {
                    user,
                    num_incorrect_pin_attempts: None,
                    has_pin: true,
                }))
            }
            _ => Err(ParseError::UnknownCardType(card_type)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("missing card type field")]
    MissingCardTypeField,
    #[error("missing jurisdiction field")]
    MissingJurisdictionField,
    #[error("missing election hash field")]
    MissingElectionHashField,
    #[error("invalid jurisdiction field: {0}")]
    InvalidJurisdictionField(String),
    #[error("openssl error: {0}")]
    OpenSSL(#[from] openssl::error::ErrorStack),
    #[error("unknown card type: {0}")]
    UnknownCardType(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemAdministratorCardDetails {
    pub user: SystemAdministratorUser,
    pub num_incorrect_pin_attempts: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElectionManagerCardDetails {
    pub user: ElectionManagerUser,
    pub num_incorrect_pin_attempts: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PollWorkerCardDetails {
    pub user: PollWorkerUser,
    pub num_incorrect_pin_attempts: Option<u8>,

    /// Unlike system administrator and election manager cards, which always
    /// have PINs, poll worker cards by default don't have PINs but can if the
    /// relevant system setting is enabled.
    pub has_pin: bool,
}
