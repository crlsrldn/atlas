use reqwest::Client;
use std::env;
use serde_json::json;

use once_cell::sync::Lazy;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

pub async fn get_preferences_from_cloud() -> Option<crate::api::config::UserPreferences> {
    let endpoint = env::var("APPWRITE_ENDPOINT").ok()?;
    let project = env::var("APPWRITE_PROJECT_ID").ok()?;
    let key = env::var("APPWRITE_API_KEY").ok()?;

    let url = format!("{}/databases/atlas/collections/preferences/documents/global_prefs", endpoint);
    
    let res = HTTP_CLIENT.get(&url)
        .header("X-Appwrite-Project", project)
        .header("X-Appwrite-Key", key)
        .send()
        .await
        .ok()?;

    if res.status().is_success() {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(prefs_str) = json["prefs_json"].as_str() {
                if let Ok(prefs) = serde_json::from_str::<crate::api::config::UserPreferences>(prefs_str) {
                    return Some(prefs);
                }
            }
        }
    }
    
    None
}

pub async fn save_preferences_to_cloud(prefs: &crate::api::config::UserPreferences) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = env::var("APPWRITE_ENDPOINT")?;
    let project = env::var("APPWRITE_PROJECT_ID")?;
    let key = env::var("APPWRITE_API_KEY")?;

    let url = format!("{}/databases/atlas/collections/preferences/documents/global_prefs", endpoint);
    
    let payload_str = serde_json::to_string(prefs).unwrap();

    // Try to update existing first (PATCH)
    let res = HTTP_CLIENT.patch(&url)
        .header("X-Appwrite-Project", &project)
        .header("X-Appwrite-Key", &key)
        .json(&json!({ "data": { "prefs_json": payload_str } }))
        .send()
        .await?;

    if res.status() == reqwest::StatusCode::NOT_FOUND {
        // Doesn't exist, create it (POST)
        let create_url = format!("{}/databases/atlas/collections/preferences/documents", endpoint);
        HTTP_CLIENT.post(&create_url)
            .header("X-Appwrite-Project", &project)
            .header("X-Appwrite-Key", &key)
            .json(&json!({
                "documentId": "global_prefs",
                "data": { "prefs_json": payload_str }
            }))
            .send()
            .await?;
    }

    Ok(())
}
