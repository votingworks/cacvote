use cacvote_server::client::Client;
use serde::{Deserialize, Serialize};
use types_rs::cacvote::{Payload, SignedObject};

#[derive(Debug, Serialize, Deserialize)]
struct TestObject {
    name: String,
    description: String,
    value: i32,
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
    let signed_object = SignedObject {
        payload: serde_json::to_vec(&payload)?,
        certificate: Vec::new(),
        signature: Vec::new(),
    };

    let client = Client::new("http://localhost:8000".parse()?);
    let object_id = client.create_object(signed_object).await?;
    println!("object_id: {object_id:?}");

    Ok(())
}
