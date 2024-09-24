pub use cacvote_server_client::{Client, Error, Result};

#[cfg(test)]
mod tests {
    use cacvote_server_client::{PrivateKeySigner, Signer};
    use openssl::{pkey::PKey, x509};
    use sqlx::PgPool;
    use types_rs::cacvote::{
        JournalEntryAction, JurisdictionCode, Payload, RegistrationRequest, SignedObject,
    };
    use uuid::Uuid;

    use super::*;
    use crate::app;

    async fn setup(
        pool: PgPool,
        vx_root_ca_cert: x509::X509,
        cac_root_ca_store: x509::store::X509Store,
    ) -> color_eyre::Result<Client> {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await?;
        let addr = listener.local_addr()?;

        tokio::spawn({
            let vx_root_ca_cert = vx_root_ca_cert.clone();
            async move {
                let app = app::setup(pool, vx_root_ca_cert, cac_root_ca_store).await;
                axum::serve(listener, app).await.unwrap();
            }
        });

        let request_signer = PrivateKeySigner::new(PKey::private_key_from_pem(include_bytes!(
            "../../../../libs/auth/certs/dev/vx-admin-private-key.pem"
        ))?);
        let signing_cert = x509::X509::from_pem(include_bytes!(
            "../../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem"
        ))?;

        Ok(Client::new(
            format!("http://{addr}").parse()?,
            signing_cert,
            Box::new(request_signer),
        ))
    }

    fn load_cryptographic_info(
    ) -> color_eyre::Result<(x509::X509, x509::store::X509Store, PrivateKeySigner)> {
        let vx_root_ca_cert = x509::X509::from_pem(include_bytes!(
            "../../../../libs/auth/certs/dev/vx-cert-authority-cert.pem"
        ))?;
        let cac_root_ca_certs = vec![
            // Sample CAC certificate
            x509::X509::from_der(include_bytes!(
                "../../../cacvote-jx-terminal/backend/certs/DODJITCEMAILCA_63.cer"
            ))?,
            // Development CAC certificate
            x509::X509::from_pem(include_bytes!(
                "../../../../libs/auth/certs/dev/vx-cert-authority-cert.pem"
            ))?,
        ];
        let mut cac_root_ca_store_builder = x509::store::X509StoreBuilder::new()?;

        for cac_root_ca_cert in cac_root_ca_certs {
            cac_root_ca_store_builder.add_cert(cac_root_ca_cert.clone())?;
        }

        let cac_root_ca_store = cac_root_ca_store_builder.build();

        let private_key_pem = include_bytes!("../../../../libs/auth/certs/dev/vx-private-key.pem");
        let private_key = PKey::private_key_from_pem(private_key_pem)?;
        let object_signer = PrivateKeySigner::new(private_key);

        Ok((vx_root_ca_cert, cac_root_ca_store, object_signer))
    }

    #[sqlx::test(migrations = "db/migrations")]
    async fn test_client(pool: sqlx::PgPool) -> color_eyre::Result<()> {
        let (vx_root_ca_cert, cac_root_ca_store, object_signer) = load_cryptographic_info()?;
        let mut client = setup(pool, vx_root_ca_cert.clone(), cac_root_ca_store).await?;

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
        let signature = object_signer.sign(&payload)?;

        // create the object
        let object_id = client
            .create_object(SignedObject {
                id: Uuid::new_v4(),
                election_id: None,
                payload,
                certificate: vx_root_ca_cert.to_pem()?,
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
        assert_eq!(signed_object.certificate, vx_root_ca_cert.to_pem()?);
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
        let (vx_root_ca_cert, cac_root_ca_store, _) = load_cryptographic_info()?;
        let mut client = setup(pool, vx_root_ca_cert, cac_root_ca_store).await?;

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
                // invalid certificate and signature
                certificate: vec![],
                signature: vec![],
            })
            .await
            .unwrap_err();

        // check that there are no journal entries
        assert_eq!(client.get_journal_entries(None, None).await?, vec![]);

        Ok(())
    }
}
