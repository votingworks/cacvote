use std::string::FromUtf8Error;

/// Error type for TLV operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid length: expected {expected}, actual {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Invalid tag: expected {expected:?}, actual {actual:?}")]
    InvalidTag { expected: Vec<u8>, actual: Vec<u8> },

    #[error("Value too long: {0} bytes > u16::MAX")]
    ValueTooLong(usize),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
}

/// Result type for TLV operations.
pub type Result<T, E = Error> = std::result::Result<T, E>;
