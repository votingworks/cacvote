use auth_rs::{card_details::extract_field_value, certs::VX_CUSTOM_CERT_FIELD_JURISDICTION};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use openssl::x509::X509;
use types_rs::cacvote;
use uuid::Uuid;

use crate::state::AppState;

/// Represents a simple user session that is kept in memory.
#[derive(Debug, Clone)]
pub(crate) struct Session {
    /// The client's signing certificate signed by the CA that `cacvote-server` trusts.
    // FIXME: do we need this?
    #[allow(dead_code)]
    certificate: X509,

    /// The jurisdiction code of the client's signing certificate.
    jurisdiction_code: cacvote::JurisdictionCode,

    /// The session token. This is meant to be opaque to the client.
    token: Uuid,

    /// The expiration time of the session.
    expiration: time::OffsetDateTime,
}

const SESSION_DURATION: time::Duration = time::Duration::minutes(15);

impl Session {
    /// Creates a new session from the given certificate.
    ///
    /// # Errors
    ///
    /// Returns an error if the jurisdiction code field is not found in the
    /// certificate or if the jurisdiction code is invalid.
    pub(crate) fn new(certificate: X509) -> Result<Self, Error> {
        let token = Uuid::new_v4();
        let expiration = time::OffsetDateTime::now_utc() + SESSION_DURATION;
        let jurisdiction_code =
            match extract_field_value(&certificate, VX_CUSTOM_CERT_FIELD_JURISDICTION) {
                Ok(Some(s)) => cacvote::JurisdictionCode::try_from(s.clone())
                    .map_err(|_| Error::InvalidJurisdictionCode(s.to_owned()))?,
                Ok(None) | Err(_) => {
                    return Err(Error::FieldNotFound(
                        VX_CUSTOM_CERT_FIELD_JURISDICTION.to_owned(),
                    ))
                }
            };

        Ok(Self {
            certificate,
            jurisdiction_code,
            token,
            expiration,
        })
    }

    /// Returns the jurisdiction code of the session.
    pub(crate) fn jurisdiction_code(&self) -> &cacvote::JurisdictionCode {
        &self.jurisdiction_code
    }

    /// Returns whether the session has expired.
    pub(crate) fn is_expired(&self) -> bool {
        time::OffsetDateTime::now_utc() > self.expiration
    }

    /// Validates the session based on the token and expiration time.
    pub(crate) fn validate(&self, token: Uuid) -> bool {
        token == self.token && !self.is_expired()
    }

    /// Returns the session token.
    pub(crate) fn token(&self) -> impl ToString {
        self.token
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("field not found: {0}")]
    FieldNotFound(String),

    #[error("invalid jurisdiction code: {0}")]
    InvalidJurisdictionCode(String),
}

/// Manages a collection of user sessions stored in memory.
#[derive(Debug)]
pub(crate) struct SessionManager {
    sessions: Vec<Session>,
}

impl SessionManager {
    /// Creates a new session manager.
    pub(crate) const fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    /// Creates a new session and returns it.
    pub(crate) fn create(&mut self, certificate: X509) -> Result<Session, Error> {
        let session = Session::new(certificate)?;
        self.sessions.push(session.clone());
        Ok(session)
    }

    /// Validates a session token and returns the session if it is valid.
    pub(crate) fn validate(&mut self, token: Uuid) -> Option<Session> {
        self.sessions.retain(|s| !s.is_expired());
        self.sessions.iter().find(|s| s.validate(token)).cloned()
    }
}

/// Extracts a session from the request's authorization header, allowing request
/// methods to require a session by including `Session` in their signature.
#[async_trait]
impl FromRequestParts<AppState> for Session {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState { sessions, .. }: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        let token = Uuid::parse_str(bearer.token()).map_err(|_| StatusCode::UNAUTHORIZED)?;

        // Expire old tokens
        let mut sessions = sessions.lock().await;

        // Look for a valid session with the given token
        if let Some(session) = sessions.validate(token) {
            tracing::debug!("Authorized session: {:?}", session.jurisdiction_code());
            Ok(session)
        } else {
            tracing::warn!("Unauthorized session: {token}");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
