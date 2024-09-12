use std::fmt::Debug;

use base64_serde::base64_serde_type;
use openssl::x509::X509;
use serde::{Deserialize, Serialize};
use types_rs::cacvote::{JournalEntry, JurisdictionCode, SignedObject};
use uuid::Uuid;

use crate::result::{Error, Result};
use crate::signer::AnySigner;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

/// A client for the CACvote server.
pub struct Client {
    base_url: reqwest::Url,

    /// The certificate wrapping the public key used to verify signatures.
    signing_cert: X509,

    /// The signer used to sign payloads.
    signer: AnySigner,

    /// The bearer token for the current session.
    bearer_token: Option<String>,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl Client {
    /// Create a new client with the given base URL.
    ///
    /// # Example
    ///
    /// ```
    /// # use cacvote_server_client::Client;
    /// let base_url = "http://localhost:8000".parse().unwrap();
    /// let client = Client::new(base_url);
    /// ```
    pub const fn new(base_url: reqwest::Url, signing_cert: X509, signer: AnySigner) -> Self {
        Self {
            base_url,
            signing_cert,
            signer,
            bearer_token: None,
        }
    }

    /// Create a new client to connect to the server running on localhost.
    pub fn localhost(signing_cert: X509, signer: AnySigner) -> Self {
        Self::new(
            "http://localhost:8000"
                .parse()
                .expect("hardcoded URL is valid"),
            signing_cert,
            signer,
        )
    }

    /// Check that the server is responding.
    pub async fn check_status(&self) -> Result<()> {
        let response = self.get("/api/status").await?;
        response.error_for_status()?;
        Ok(())
    }

    /// Authenticate with the server. Creates a new session and stores the
    /// bearer token for future requests.
    pub async fn authenticate(&mut self) -> Result<()> {
        let certificates = self
            .signing_cert
            .to_pem()
            .map_err(|e| Error::Signature(format!("failed to serialize signing cert: {e}")))?;
        let payload = CreateSessionRequestPayload {
            timestamp: time::OffsetDateTime::now_utc(),
        };
        let payload = serde_json::to_vec(&payload)?;
        let signature = self
            .signer
            .sign(&payload)
            .map_err(|e| Error::Signature(format!("failed to sign payload: {e}")))?;
        let request = CreateSessionRequest {
            certificate: certificates,
            payload,
            signature,
        };
        let response = self.post_json("/api/sessions", &request).await?;
        let response: CreateSessionResponse = response.error_for_status()?.json().await?;
        self.bearer_token = Some(response.bearer_token);
        Ok(())
    }

    /// Create an object on the server.
    pub async fn create_object(&mut self, signed_object: SignedObject) -> Result<Uuid> {
        loop {
            self.authenticate_if_needed().await?;
            let response = self.post_json("/api/objects", &signed_object).await?;

            if let reqwest::StatusCode::UNAUTHORIZED = response.status() {
                self.bearer_token = None;
                continue;
            }
            let status_code = response.status();
            let text = response.text().await?;

            if !status_code.is_success() {
                return Err(Error::Http {
                    status_code,
                    text,
                    context: format!("failed to create object with ID {:?}", signed_object.id),
                });
            }

            return Ok(Uuid::try_parse(&text)?);
        }
    }

    /// Get an object by its ID.
    ///
    /// # Example
    ///
    /// ```
    /// # use cacvote_server_client::Client;
    /// # async {
    /// # let client = Client::localhost();
    /// let object_id = client
    ///     .get_object_by_id("00000000-0000-0000-0000-000000000000".parse().unwrap())
    ///     .await
    ///     .unwrap();
    /// # };
    /// ```
    pub async fn get_object_by_id(&mut self, object_id: Uuid) -> Result<Option<SignedObject>> {
        let path = format!("/api/objects/{object_id}");
        loop {
            self.authenticate_if_needed().await?;
            let response = self.get(&path).await?;

            match response.status() {
                reqwest::StatusCode::NOT_FOUND => return Ok(None),
                reqwest::StatusCode::UNAUTHORIZED => {
                    self.bearer_token = None;
                    continue;
                }
                status_code if status_code.is_success() => {
                    return Ok(Some(response.json().await?));
                }
                status_code => {
                    return Err(Error::Http {
                        status_code,
                        text: response.text().await?,
                        context: format!("failed to get object by ID {object_id:?}"),
                    });
                }
            }
        }
    }

    /// Get journal entries from the server.
    ///
    /// # Example
    ///
    /// ```
    /// # use cacvote_server_client::Client;
    /// # use types_rs::cacvote::JurisdictionCode;
    /// # async {
    /// # let client = Client::localhost();
    /// // get all journal entries ever
    /// let entries = client.get_journal_entries(None, None).await.unwrap();
    ///
    /// // get all journal entries since a specific entry
    /// let entries = client.get_journal_entries(
    ///     Some(&"00000000-0000-0000-0000-000000000000".parse().unwrap()),
    ///     None,
    /// ).await.unwrap();
    ///
    /// // get all journal entries for a specific jurisdiction
    /// let entries = client.get_journal_entries(
    ///     None,
    ///     Some(&JurisdictionCode::try_from("st.dev-jurisdiction").unwrap()),
    /// ).await.unwrap();
    /// # };
    /// ```
    pub async fn get_journal_entries(
        &mut self,
        since: Option<&Uuid>,
        jurisdiction_code: Option<&JurisdictionCode>,
    ) -> Result<Vec<JournalEntry>> {
        let mut params = Vec::new();

        if let Some(since) = since {
            params.push(("since", since.to_string()));
        }

        if let Some(jurisdiction_code) = jurisdiction_code {
            params.push(("jurisdiction", jurisdiction_code.to_string()));
        }

        let url = reqwest::Url::parse_with_params(
            self.base_url.join("/api/journal-entries")?.as_str(),
            params,
        )?;

        loop {
            self.authenticate_if_needed().await?;
            let response = self.get(url.as_str()).await?;

            match response.status() {
                reqwest::StatusCode::UNAUTHORIZED => {
                    self.bearer_token = None;
                    continue;
                }
                status_code if status_code.is_success() => {
                    return Ok(response.json().await?);
                }
                status_code => {
                    return Err(Error::Http {
                        status_code,
                        text: response.text().await?,
                        context: format!(
                            "failed to get journal entries since={since:?} jurisdiction={jurisdiction_code:?}"
                        )
                    });
                }
            }
        }
    }

    fn normalize_url(&self, path: &str) -> Result<reqwest::Url> {
        if path.starts_with(self.base_url.as_str()) {
            Ok(reqwest::Url::parse(path)?)
        } else {
            Ok(self.base_url.join(path)?)
        }
    }

    async fn authenticate_if_needed(&mut self) -> Result<()> {
        if self.bearer_token.is_none() {
            self.authenticate().await?;
        }
        Ok(())
    }

    async fn get(&self, path: &str) -> Result<reqwest::Response> {
        let request = self.make_request(reqwest::Method::GET, path)?;
        Ok(request.send().await?)
    }

    async fn post_json(&self, path: &str, body: &impl Serialize) -> Result<reqwest::Response> {
        let request = self.make_request(reqwest::Method::POST, path)?;
        Ok(request.json(body).send().await?)
    }

    fn make_request(&self, method: reqwest::Method, path: &str) -> Result<reqwest::RequestBuilder> {
        let mut request = reqwest::Client::new().request(method, self.normalize_url(path)?);

        if let Some(ref bearer_token) = self.bearer_token {
            request = request.bearer_auth(bearer_token);
        }

        Ok(request)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    /// A PEM-encoded X.509 certificate. Contains the client TPM's public key
    /// certificate as signed by the CA given in the cacvote-server
    /// configuration.
    #[serde(with = "Base64Standard")]
    pub certificate: Vec<u8>,

    /// The payload of the request. Must be JSON decodable as
    /// [`CreateSessionRequestPayload`][CreateSessionRequestPayload].
    #[serde(with = "Base64Standard")]
    pub payload: Vec<u8>,

    /// The signature of the payload as signed by the client's TPM.
    #[serde(with = "Base64Standard")]
    pub signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequestPayload {
    #[serde(with = "time::serde::iso8601")]
    pub timestamp: time::OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub bearer_token: String,
}
