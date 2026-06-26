use std::env;
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let endpoint = env::var("APPWRITE_ENDPOINT").expect("APPWRITE_ENDPOINT not set");
    let project = env::var("APPWRITE_PROJECT_ID").expect("APPWRITE_PROJECT_ID not set");
    let key = env::var("APPWRITE_API_KEY").expect("APPWRITE_API_KEY not set");

    let client = Client::new();

    println!("Provisioning Appwrite Database...");

    // 1. Create Database
    let url = format!("{}/databases", endpoint);
    let res = client.post(&url)
        .header("X-Appwrite-Project", &project)
        .header("X-Appwrite-Key", &key)
        .json(&json!({
            "databaseId": "atlas",
            "name": "Atlas"
        }))
        .send()
        .await
        .unwrap();
    println!("Create Database status: {}", res.status());

    // 2. Create Preferences Collection
    let url = format!("{}/databases/atlas/collections", endpoint);
    let res = client.post(&url)
        .header("X-Appwrite-Project", &project)
        .header("X-Appwrite-Key", &key)
        .json(&json!({
            "collectionId": "preferences",
            "name": "Preferences",
            "documentSecurity": false
        }))
        .send()
        .await
        .unwrap();
    println!("Create Prefs Collection status: {}", res.status());

    // 3. Create Preferences prefs_json Attribute
    let url = format!("{}/databases/atlas/collections/preferences/attributes/string", endpoint);
    let res = client.post(&url)
        .header("X-Appwrite-Project", &project)
        .header("X-Appwrite-Key", &key)
        .json(&json!({
            "key": "prefs_json",
            "size": 5000,
            "required": false
        }))
        .send()
        .await
        .unwrap();
    println!("Create Attr prefs_json status: {}", res.status());

    // 4. Create Telemetry Collection
    let url = format!("{}/databases/atlas/collections", endpoint);
    let res = client.post(&url)
        .header("X-Appwrite-Project", &project)
        .header("X-Appwrite-Key", &key)
        .json(&json!({
            "collectionId": "telemetry",
            "name": "Telemetry",
            "documentSecurity": false
        }))
        .send()
        .await
        .unwrap();
    println!("Create Telemetry Collection status: {}", res.status());

    // 5. Create Telemetry telemetry_json Attribute
    let url = format!("{}/databases/atlas/collections/telemetry/attributes/string", endpoint);
    let res = client.post(&url)
        .header("X-Appwrite-Project", &project)
        .header("X-Appwrite-Key", &key)
        .json(&json!({
            "key": "telemetry_json",
            "size": 5000,
            "required": false
        }))
        .send()
        .await
        .unwrap();
    println!("Create Attr telemetry_json status: {}", res.status());
}
