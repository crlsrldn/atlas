use reqwest::Client;
use serde_json::json;
use serde_json::Value;
use std::env;

use once_cell::sync::Lazy;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

pub async fn get_preferences_from_cloud() -> Option<crate::api::config::UserPreferences> {
    let endpoint = env::var("APPWRITE_ENDPOINT").ok()?;
    let project = env::var("APPWRITE_PROJECT_ID").ok()?;
    let key = env::var("APPWRITE_API_KEY").ok()?;

    let url = format!(
        "{}/databases/atlas/collections/preferences/documents/global_prefs",
        endpoint
    );

    let res = HTTP_CLIENT
        .get(&url)
        .header("X-Appwrite-Project", project)
        .header("X-Appwrite-Key", key)
        .send()
        .await
        .ok()?;

    if res.status().is_success() {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(prefs_str) = json["prefs_json"].as_str() {
                if let Ok(prefs) =
                    serde_json::from_str::<crate::api::config::UserPreferences>(prefs_str)
                {
                    return Some(prefs);
                }
            }
        }
    }

    None
}

pub async fn save_preferences_to_cloud(
    prefs: &crate::api::config::UserPreferences,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = env::var("APPWRITE_ENDPOINT")?;
    let project = env::var("APPWRITE_PROJECT_ID")?;
    let key = env::var("APPWRITE_API_KEY")?;

    let url = format!(
        "{}/databases/atlas/collections/preferences/documents/global_prefs",
        endpoint
    );

    let payload_str = serde_json::to_string(prefs).unwrap();

    // Try to update existing first (PATCH)
    let res = HTTP_CLIENT
        .patch(&url)
        .header("X-Appwrite-Project", &project)
        .header("X-Appwrite-Key", &key)
        .json(&json!({ "data": { "prefs_json": payload_str } }))
        .send()
        .await?;

    if res.status() == reqwest::StatusCode::NOT_FOUND {
        // Doesn't exist, create it (POST)
        let create_url = format!(
            "{}/databases/atlas/collections/preferences/documents",
            endpoint
        );
        HTTP_CLIENT
            .post(&create_url)
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

pub async fn get_recent_telemetry(limit: usize) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let endpoint = env::var("APPWRITE_ENDPOINT")?;
    let project = env::var("APPWRITE_PROJECT_ID")?;
    let key = env::var("APPWRITE_API_KEY")?;

    let url = format!(
        "{}/databases/atlas/collections/telemetry/documents",
        endpoint
    );

    let limit_query = format!("limit({})", limit.clamp(1, 100));
    let response = HTTP_CLIENT
        .get(&url)
        .header("X-Appwrite-Project", &project)
        .header("X-Appwrite-Key", &key)
        .query(&[
            ("queries[]", "orderDesc(\"$createdAt\")"),
            ("queries[]", limit_query.as_str()),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Appwrite telemetry request failed: {}", response.status()).into());
    }

    let json = response.json::<Value>().await?;
    let documents = json
        .get("documents")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    Ok(documents
        .into_iter()
        .filter_map(|document| {
            document
                .get("telemetry_json")
                .and_then(Value::as_str)
                .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        })
        .collect())
}
