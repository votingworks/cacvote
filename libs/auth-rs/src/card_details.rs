use openssl::x509::X509;
use types_rs::auth::{ElectionManagerUser, PollWorkerUser, SystemAdministratorUser};

pub enum CardDetails {
    SystemAdministrator(SystemAdministratorCardDetails),
    ElectionManager(ElectionManagerCardDetails),
    PollWorker(PollWorkerCardDetails),
}

impl From<X509> for CardDetails {
    fn from(value: X509) -> Self {}
}

pub struct SystemAdministratorCardDetails {
    user: SystemAdministratorUser,
    num_incorrect_pin_attempts: Option<u8>,
}

pub struct ElectionManagerCardDetails {
    user: ElectionManagerUser,
    num_incorrect_pin_attempts: Option<u8>,
}

pub struct PollWorkerCardDetails {
    user: PollWorkerUser,
    num_incorrect_pin_attempts: Option<u8>,

    /// Unlike system administrator and election manager cards, which always
    /// have PINs, poll worker cards by default don't have PINs but can if the
    /// relevant system setting is enabled.
    has_pin: bool,
}
