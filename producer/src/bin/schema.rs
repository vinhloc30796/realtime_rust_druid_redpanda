// Reference: https://druid.apache.org/docs/latest/development/extensions-core/protobuf.html#when-using-schema-registry-1

// Serialization
use serde_json::json;
// Import post_request & get_request from local lib.rs
use producer::{post_request, get_request};


#[tokio::main]
async fn main() {
    // Logging config with env_logger
    env_logger::init();
    
    // Read proto file from protos/hackernews.proto
    let proto_filename = "src/protos/hackernews.proto";
    let proto_string = std::fs::read_to_string(proto_filename).unwrap();
    let proto_body = json!({
        "schemaType": "PROTOBUF",
        "schema": proto_string
    });

    // Make the requests
    let uri = "http://localhost:8081/subjects/hackernews-topic-value/versions";
    post_request(uri, proto_body).await; // POST request to create a new schema
    get_request(uri).await; // GET request to get the schema
}