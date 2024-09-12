use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::session::SessionManager;

/// Contains the application state, used by request handlers.
#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub pool: PgPool,
    pub ca_cert: openssl::x509::X509,
    pub sessions: Arc<Mutex<SessionManager>>,
}
