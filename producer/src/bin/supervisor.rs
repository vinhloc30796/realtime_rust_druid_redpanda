// Reference: https://druid.apache.org/docs/25.0.0/operations/api-reference.html#supervisors
// Reference: https://github.com/wiikip/road-traffic-analytics/blob/main/deploy.sh#L34

// Serialization
use serde_json::from_str;
// Import post_request & get_request from local lib.rs
use producer::{post_request, get_request};

#[tokio::main]
async fn main() {
    // Logging config with env_logger
    env_logger::init();
    
    let json_filename = "src/supervisors/hackernews-supervisor.json";
    let json_string = std::fs::read_to_string(json_filename).unwrap();
    let json_body: serde_json::Value = from_str(&json_string).unwrap();
    
    // Make the requests
    let uri = "http://localhost:28081/druid/indexer/v1/supervisor";
    post_request(uri, json_body).await; // POST request to create a new schema
    get_request(uri).await; // GET request to get the schema
}