pub use cacvote_server_client::{Client, Error, Result};

#[cfg(test)]
mod tests {
    use openssl::{
        hash::MessageDigest,
        pkey::{PKey, Private, Public},
        sign::{Signer, Verifier},
        x509::X509,
    };
    use types_rs::cacvote::{
        JournalEntryAction, JurisdictionCode, Payload, RegistrationRequest, SignedObject,
    };
    use uuid::Uuid;

    use super::*;
    use crate::app;

    struct TestSigner {
        private_key: PKey<Private>,
    }

    impl cacvote_server_client::Signer for TestSigner {
        fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
            let mut signer = Signer::new(MessageDigest::sha256(), &self.private_key)
                .map_err(|e| Error::Signature(e.to_string()))?;
            signer
                .update(payload)
                .map_err(|e| Error::Signature(e.to_string()))?;
            signer
                .sign_to_vec()
                .map_err(|e| Error::Signature(e.to_string()))
        }
    }

    async fn setup(pool: sqlx::PgPool, ca_cert: openssl::x509::X509) -> color_eyre::Result<Client> {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            let app = app::setup(pool, ca_cert).await;
            axum::serve(listener, app).await.unwrap();
        });

        Ok(Client::new(
            format!("http://{addr}").parse()?,
            X509::from_pem(include_bytes!(
                "../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem"
            ))?,
            Box::new(TestSigner {
                private_key: PKey::private_key_from_pem(include_bytes!(
                    "../../../../libs/auth/certs/dev/vx-admin-private-key.pem"
                ))?,
            }),
        ))
    }

    fn load_keypair() -> color_eyre::Result<(Vec<u8>, PKey<Public>, PKey<Private>)> {
        // uses the dev VxAdmin keypair because it has the Jurisdiction field
        let private_key_pem =
            include_bytes!("../../../../libs/auth/certs/dev/vx-admin-private-key.pem");
        let private_key = PKey::private_key_from_pem(private_key_pem)?;
        let certificates =
            include_bytes!("../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem")
                .to_vec();
        let x509 = X509::from_pem(&certificates)?;
        let public_key = x509.public_key()?;
        Ok((certificates, public_key, private_key))
    }

    fn sign_and_verify(
        payload: &[u8],
        private_key: &PKey<Private>,
        public_key: &PKey<Public>,
    ) -> color_eyre::Result<Vec<u8>> {
        let mut signer = Signer::new(MessageDigest::sha256(), private_key)?;
        signer.update(payload)?;
        let signature = signer.sign_to_vec()?;

        let mut verifier = Verifier::new(MessageDigest::sha256(), public_key)?;
        verifier.update(payload)?;
        assert!(verifier.verify(&signature)?);
        Ok(signature)
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_client(pool: sqlx::PgPool) -> color_eyre::Result<()> {
        let ca_cert = openssl::x509::X509::from_pem(include_bytes!(
            "../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem"
        ))?;
        let mut client = setup(pool, ca_cert).await?;

        client.authenticate().await?;

        let entries = client.get_journal_entries(None, None).await?;
        assert_eq!(entries, vec![]);

        let payload = Payload::RegistrationRequest(RegistrationRequest {
            common_access_card_id: "1234567890".to_owned(),
            given_name: "John".to_owned(),
            family_name: "Doe".to_owned(),
            jurisdiction_code: JurisdictionCode::try_from("st.dev-jurisdiction").unwrap(),
        });
        let payload = serde_json::to_vec(&payload)?;
        let (certificates, public_key, private_key) = load_keypair()?;
        let signature = sign_and_verify(&payload, &private_key, &public_key)?;

        // create the object
        let object_id = client
            .create_object(SignedObject {
                id: Uuid::new_v4(),
                election_id: None,
                payload,
                certificates: certificates.clone(),
                signature: signature.clone(),
            })
            .await?;

        // check the journal
        let entries = client.get_journal_entries(None, None).await?;
        let entry = match entries.as_slice() {
            [entry] => {
                assert_eq!(entry.object_id, object_id);
                assert_eq!(entry.action, JournalEntryAction::Create);
                assert_eq!(entry.object_type, "RegistrationRequest");
                assert_eq!(entry.jurisdiction_code.as_str(), "st.dev-jurisdiction");
                entry
            }
            _ => panic!("expected one journal entry, got: {entries:?}"),
        };

        // filter by jurisdiction code
        assert_eq!(
            client
                .get_journal_entries(
                    None,
                    Some(&JurisdictionCode::try_from("st.dev-jurisdiction").unwrap())
                )
                .await?,
            vec![entry.clone()]
        );
        assert_eq!(
            client
                .get_journal_entries(
                    None,
                    Some(&JurisdictionCode::try_from("st.other-jurisdiction").unwrap())
                )
                .await?,
            vec![]
        );

        // check the journal since the last entry
        assert_eq!(
            client.get_journal_entries(Some(&entry.id), None).await?,
            vec![]
        );

        // get the object
        let signed_object = client.get_object_by_id(object_id).await?.unwrap();

        let round_trip_registration_request = match signed_object.try_to_inner()? {
            Payload::RegistrationRequest(registration_request) => registration_request,
            other => panic!("expected RegistrationRequest, got: {other:?}"),
        };
        assert_eq!(signed_object.certificates, certificates);
        assert_eq!(signed_object.signature, signature);
        assert_eq!(
            round_trip_registration_request.common_access_card_id,
            "1234567890"
        );
        assert_eq!(round_trip_registration_request.given_name, "John");
        assert_eq!(round_trip_registration_request.family_name, "Doe");
        assert_eq!(
            round_trip_registration_request.jurisdiction_code.as_str(),
            "st.dev-jurisdiction"
        );

        Ok(())
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_invalid_certificate(pool: sqlx::PgPool) -> color_eyre::Result<()> {
        let ca_cert = openssl::x509::X509::from_pem(include_bytes!(
            "../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem"
        ))?;
        let mut client = setup(pool, ca_cert).await?;

        let payload = Payload::RegistrationRequest(RegistrationRequest {
            common_access_card_id: "1234567890".to_owned(),
            given_name: "John".to_owned(),
            family_name: "Doe".to_owned(),
            jurisdiction_code: JurisdictionCode::try_from("st.dev-jurisdiction").unwrap(),
        });
        let payload = serde_json::to_vec(&payload)?;

        client
            .create_object(SignedObject {
                id: Uuid::new_v4(),
                payload,
                election_id: None,
                // invalid certificates and signature
                certificates: vec![],
                signature: vec![],
            })
            .await
            .unwrap_err();

        // check that there are no journal entries
        assert_eq!(client.get_journal_entries(None, None).await?, vec![]);

        Ok(())
    }
}
