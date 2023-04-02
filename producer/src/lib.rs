
// IO
use log::{info, error};

pub async fn get_request(url: &str) {
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

pub async fn post_request(
    url: &str,
    body: serde_json::Value
) {
    let client = reqwest::Client::new();
    info!("Body: {:#?}", body);
    // Post with header
    let req = client
        .post(url)
        // .header("Content-Type", "application/vnd.schemaregistry.v1+json")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;
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