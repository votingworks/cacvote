use reqwest::{Response, Url};
use types_rs::cacvote::{JournalEntry, SignedObject};
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

/// A client for the CACVote server.
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
    /// # use cacvote_server::client::Client;
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
    /// # use cacvote_server::client::Client;
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
    /// # use cacvote_server::client::Client;
    /// # async {
    /// # let client = Client::localhost();
    /// // get all journal entries ever
    /// let entries = client.get_journal_entries(None).await.unwrap();
    ///
    /// // get all journal entries since a specific entry
    /// let entries = client.get_journal_entries(Some("00000000-0000-0000-0000-000000000000".parse().unwrap())).await.unwrap();
    /// # };
    /// ```
    pub async fn get_journal_entries(&self, since: Option<Uuid>) -> Result<Vec<JournalEntry>> {
        let params = match since {
            Some(since) => vec![("since", since.to_string())],
            None => vec![],
        };
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

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use serde_json::json;
    use types_rs::cacvote::Payload;

    use super::*;
    use crate::app;

    fn setup(pool: sqlx::PgPool) -> color_eyre::Result<Client> {
        let listener = TcpListener::bind("0.0.0.0:0")?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            let app = app::setup(pool).await.unwrap();
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        Ok(Client::new(format!("http://{addr}").parse()?))
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_client(pool: sqlx::PgPool) -> color_eyre::Result<()> {
        let client = setup(pool)?;

        let entries = client.get_journal_entries(None).await?;
        assert_eq!(entries, vec![]);

        let payload = Payload {
            data: serde_json::to_vec(&json!({ "hello": "world" }))?,
            object_type: "test".to_owned(),
        };

        // create the object
        let object_id = client
            .create_object(SignedObject {
                payload: serde_json::to_vec(&payload)?,
                certificate: vec![],
                signature: vec![],
            })
            .await?;

        // check the journal
        let entries = client.get_journal_entries(None).await?;
        let entry = match entries.as_slice() {
            [entry] => {
                assert_eq!(entry.object_id, object_id);
                assert_eq!(entry.action, "create");
                assert_eq!(entry.object_type, "test");
                // TODO: check jurisdiction matches certificate
                entry
            }
            _ => panic!("expected one journal entry, got: {entries:?}"),
        };

        // check the journal since the last entry
        let journal_entry_id = entry.id;
        assert_eq!(
            client.get_journal_entries(Some(journal_entry_id)).await?,
            vec![]
        );

        // get the object
        let signed_object = client.get_object_by_id(object_id).await?.unwrap();

        let round_trip_payload = signed_object.try_to_inner()?;
        let round_trip_payload_data: serde_json::Value =
            serde_json::from_slice(&round_trip_payload.data)?;
        assert!(signed_object.certificate.is_empty());
        assert!(signed_object.signature.is_empty());
        assert_eq!(round_trip_payload_data, json!({ "hello": "world" }));

        Ok(())
    }
}
