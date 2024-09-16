pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("HTTP error: {context} status_code={status_code:?} {text}")]
    Http {
        status_code: reqwest::StatusCode,
        text: String,
        context: String,
    },

    #[error("url error: {0}")]
    Url(#[from] url::ParseError),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("uuid error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("signature error: {0}")]
    Signature(String),
}
