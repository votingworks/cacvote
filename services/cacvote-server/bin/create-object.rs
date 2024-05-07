use cacvote_server::client::Client;
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private, Public},
    sign::{Signer, Verifier},
    x509::X509,
};
use serde::{Deserialize, Serialize};
use types_rs::cacvote::{JurisdictionCode, Payload, RegistrationRequest, SignedObject};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct TestObject {
    name: String,
    description: String,
    value: i32,
}

fn load_keypair() -> color_eyre::Result<(Vec<u8>, PKey<Public>, PKey<Private>)> {
    // uses the dev VxAdmin keypair because it has the Jurisdiction field
    let private_key_pem = include_bytes!("../../../libs/auth/certs/dev/vx-admin-private-key.pem");
    let private_key = PKey::private_key_from_pem(private_key_pem)?;
    let certificates =
        include_bytes!("../../../libs/auth/certs/dev/vx-admin-cert-authority-cert.pem").to_vec();
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

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let payload = Payload::RegistrationRequest(RegistrationRequest {
        common_access_card_id: "1234567890".to_owned(),
        given_name: "John".to_owned(),
        family_name: "Doe".to_owned(),
        jurisdiction_code: JurisdictionCode::try_from("st.dev-jurisdiction").unwrap(),
    });
    let (certificates, public_key, private_key) = load_keypair()?;
    let payload = serde_json::to_vec(&payload)?;
    let signature = sign_and_verify(&payload, &private_key, &public_key)?;
    let signed_object = SignedObject {
        id: Uuid::new_v4(),
        election_id: None,
        payload,
        certificates,
        signature,
    };

    let client = Client::localhost();
    let object_id = client.create_object(signed_object).await?;
    println!("object_id: {object_id:?}");

    Ok(())
}
