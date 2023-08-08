use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

pub mod client;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ServerId(Uuid);

impl ServerId {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for ServerId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for ServerId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[cfg(feature = "sqlx")]
impl sqlx::Type<sqlx::Postgres> for ServerId {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

#[cfg(feature = "sqlx")]
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ServerId {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        <Uuid as sqlx::Decode<sqlx::Postgres>>::decode(value).map(Self)
    }
}

#[cfg(feature = "sqlx")]
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ServerId {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        <Uuid as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.0, buf)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ClientId(Uuid);

impl ClientId {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for ClientId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[cfg(feature = "sqlx")]
impl sqlx::Type<sqlx::Postgres> for ClientId {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

#[cfg(feature = "sqlx")]
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ClientId {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        <Uuid as sqlx::Decode<sqlx::Postgres>>::decode(value).map(Self)
    }
}

#[cfg(feature = "sqlx")]
impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ClientId {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        <Uuid as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.0, buf)
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RaveServerSyncInput {
    #[serde(default)]
    pub last_synced_registration_request_id: Option<ServerId>,
    #[serde(default)]
    pub last_synced_registration_id: Option<ServerId>,
    #[serde(default)]
    pub last_synced_election_id: Option<ServerId>,
    #[serde(default)]
    pub last_synced_printed_ballot_id: Option<ServerId>,
    #[serde(default)]
    pub last_synced_scanned_ballot_id: Option<ServerId>,
    #[serde(default)]
    pub registration_requests: Vec<client::input::RegistrationRequest>,
    #[serde(default)]
    pub elections: Vec<client::input::Election>,
    #[serde(default)]
    pub registrations: Vec<client::input::Registration>,
    #[serde(default)]
    pub printed_ballots: Vec<client::input::PrintedBallot>,
    #[serde(default)]
    pub scanned_ballots: Vec<client::input::ScannedBallot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaveServerSyncOutput {
    pub admins: Vec<client::output::Admin>,
    pub elections: Vec<client::output::Election>,
    pub registration_requests: Vec<client::output::RegistrationRequest>,
    pub registrations: Vec<client::output::Registration>,
    pub printed_ballots: Vec<client::output::PrintedBallot>,
    pub scanned_ballots: Vec<client::output::ScannedBallot>,
}
