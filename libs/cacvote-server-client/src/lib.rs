use reqwest::{Response, Url};
use types_rs::cacvote::{JournalEntry, JurisdictionCode, SignedObject};
use uuid::Uuid;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("url error: {0}")]
    Url(#[from] url::ParseError),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("uuid error: {0}")]
    Uuid(#[from] uuid::Error),
}

/// A client for the CACvote server.
#[derive(Debug)]
pub struct Client {
    base_url: Url,
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
    pub const fn new(base_url: Url) -> Self {
        Self { base_url }
    }

    /// Create a new client to connect to the server running on localhost.
    pub fn localhost() -> Self {
        Self::new(
            "http://localhost:8000"
                .parse()
                .expect("hardcoded URL is valid"),
        )
    }

    /// Check that the server is responding.
    pub async fn check_status(&self) -> Result<()> {
        let response = self.get("/api/status").await?;
        response.error_for_status()?;
        Ok(())
    }

    /// Create an object on the server.
    pub async fn create_object(&self, signed_object: SignedObject) -> Result<Uuid> {
        let response = self.post_json("/api/objects", &signed_object).await?;
        Ok(Uuid::try_parse(&response.text().await?)?)
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
    pub async fn get_object_by_id(&self, object_id: Uuid) -> Result<Option<SignedObject>> {
        let response = self.get(&format!("/api/objects/{object_id}")).await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Ok(Some(response.error_for_status()?.json().await?))
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
        &self,
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

        let url =
            Url::parse_with_params(self.base_url.join("/api/journal-entries")?.as_str(), params)?;
        Ok(self
            .get(url.as_str())
            .await?
            .error_for_status()?
            .json::<Vec<JournalEntry>>()
            .await?)
    }

    async fn get(&self, path: &str) -> Result<Response> {
        let url = if path.starts_with(self.base_url.as_str()) {
            Url::parse(path)?
        } else {
            self.base_url.join(path)?
        };
        Ok(reqwest::get(url).await?)
    }

    async fn post_json(&self, path: &str, body: &impl serde::Serialize) -> Result<Response> {
        let url = self.base_url.join(path)?;
        Ok(reqwest::Client::new()
            .post(url)
            .json(body)
            .send()
            .await?
            .error_for_status()?)
    }
}
