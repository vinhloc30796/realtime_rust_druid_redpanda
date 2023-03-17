// Reference: https://druid.apache.org/docs/latest/development/extensions-core/protobuf.html#when-using-schema-registry-1

// IO
use log::{info, error};
// Serialization
use serde_json::json;


#[tokio::main]
async fn main() {
    // Logging config with env_logger
    env_logger::init();
    
    // Read proto file from protos/hackernews.proto
    let proto_filename = "src/protos/hackernews.proto";
    let proto_string = std::fs::read_to_string(proto_filename).unwrap();
    // Make proto_body = JSON {"schemaType": "PROTOBUF", "schema": [proto_text]}
    let proto_body = json!({
        "schemaType": "PROTOBUF",
        "schema": proto_string
    });

    // Make the requests
    let uri = "http://localhost:8081/subjects/hackernews-value/versions";
    post_request(uri, proto_body).await; // POST request to create a new schema
    get_request(uri).await; // GET request to get the schema

}

async fn get_request(url: &str) {
    let client = reqwest::Client::new();
    let req = client.get(url).send().await;
    match req {
        Ok(res) => {
            let res_text = res.text().await.unwrap();
            info!("Response: {:#?}", res_text);
        }
        Err(e) => {
            error!("Error: {:#?}", e);
        }
    }
}

async fn post_request(
    url: &str,
    body: serde_json::Value,
) {
    let client = reqwest::Client::new();
    let req = client.post(url).json(&body).send().await;
    match req {
        Ok(res) => {
            let res_text = res.text().await.unwrap();
            info!("Response: {:#?}", res_text);
        }
        Err(e) => {
            error!("Error: {:#?}", e);
        }
    }
}