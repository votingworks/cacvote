use std::{fmt, sync::Arc};

use crate::{
    async_card, card_details,
    tlv::{ConstructError, ParseError},
};

/// The OpenFIPS201 applet ID
pub(crate) const OPEN_FIPS_201_AID: [u8; 11] = [
    0xa0, 0x00, 0x00, 0x03, 0x08, 0x00, 0x00, 0x10, 0x00, 0x01, 0x00,
];

pub struct CertObject {
    pub private_key_id: u8,
}

impl CertObject {
    #[must_use]
    pub const fn new(private_key_id: u8) -> Self {
        Self { private_key_id }
    }

    /// Data object IDs of the format 0x5f 0xc1 0xXX are a PIV convention.
    #[must_use]
    pub fn object_id(&self) -> Vec<u8> {
        vec![0x5f, 0xc1, self.private_key_id]
    }
}

impl fmt::Debug for CertObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CertObject")
            .field(
                "private_key_id",
                &format_args!("{:#02x}", self.private_key_id),
            )
            .finish()
    }
}

pub type SharedCardReaders = Arc<tokio::sync::Mutex<Vec<String>>>;

#[derive(Debug, Clone)]
pub enum Event {
    ReaderAdded { reader_name: String },
    ReaderRemoved { reader_name: String },
    CardInserted { reader_name: String },
    CardRemoved { reader_name: String },
}

#[derive(Debug, thiserror::Error)]
pub enum CardReaderError {
    #[error("no card found")]
    NoCardFound,
    #[error("construct error: {0}")]
    Construct(#[from] ConstructError),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("pc/sc error: {0}")]
    Pcsc(#[from] pcsc::Error),
    #[error("APDU response error: [{sw1}, {sw2}]")]
    ApduResponse { sw1: u8, sw2: u8 },
    #[error("openssl error: {0}")]
    OpenSSL(#[from] openssl::error::ErrorStack),
    #[error("card details error: {0}")]
    CardDetails(#[from] card_details::ParseError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("certificate validation error: {0}")]
    CertificateValidation(String),
    #[error("tokio error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("{0}")]
    AsyncCard(#[from] async_card::Error),
    #[error("{0}")]
    Other(String),
}

impl CardReaderError {
    pub fn is_incorrect_pin_error(&self) -> bool {
        matches!(self, Self::ApduResponse { sw1: 0x63, .. })
    }
}
