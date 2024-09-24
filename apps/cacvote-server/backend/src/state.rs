use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::session::SessionManager;

/// Contains the application state, used by request handlers.
#[derive(Clone)]
pub(crate) struct AppState {
    /// Postgres connection pool.
    pub pool: PgPool,

    /// Certificate authority used to sign the machine certificates.
    pub vx_root_ca_cert: openssl::x509::X509,

    /// Certificate authority certificate store, used to validate the client
    /// certificates containing a CAC's public key.
    pub cac_root_ca_store: Arc<openssl::x509::store::X509Store>,

    /// In-memory session manager.
    pub sessions: Arc<Mutex<SessionManager>>,
}
