use cacvote_server::client::Client;
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private, Public},
    sign::{Signer, Verifier},
    x509::X509,
};
use serde::{Deserialize, Serialize};
use types_rs::cacvote::{Payload, SignedObject};

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
    signer.update(&payload)?;
    let signature = signer.sign_to_vec()?;

    let mut verifier = Verifier::new(MessageDigest::sha256(), public_key)?;
    verifier.update(&payload)?;
    assert!(verifier.verify(&signature)?);
    Ok(signature)
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let object = TestObject {
        name: "Test Object".to_string(),
        description: "This is a test object".to_string(),
        value: 42,
    };

    let payload = Payload {
        data: serde_json::to_vec(&object)?,
        object_type: "TestObject".to_string(),
    };
    let (certificates, public_key, private_key) = load_keypair()?;
    let payload = serde_json::to_vec(&payload)?;
    let signature = sign_and_verify(&payload, &private_key, &public_key)?;
    let signed_object = SignedObject {
        payload,
        certificates,
        signature,
    };

    let client = Client::new("http://localhost:8000".parse()?);
    let object_id = client.create_object(signed_object).await?;
    println!("object_id: {object_id:?}");

    Ok(())
}
