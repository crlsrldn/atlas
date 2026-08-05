use serde_json::json;
use serde_json::Value;
use std::env;

use crate::engines::http::client as http_client;

// Note: For MVP, global prefs uses a hardcoded UUID or we skip it.
// The Deno Dashboard saves per-user preferences with their UUID.
// For global config, we'll use a specific dummy UUID `00000000-0000-0000-0000-000000000000`
const GLOBAL_PREFS_ID: &str = "00000000-0000-0000-0000-000000000000";

pub async fn get_preferences_from_cloud() -> Option<crate::api::config::UserPreferences> {
    let endpoint = env::var("SUPABASE_URL").ok()?;
    let key = env::var("SUPABASE_SERVICE_ROLE_KEY").ok()?;

    let url = format!(
        "{}/rest/v1/preferences?id=eq.{}&select=prefs_json",
        endpoint, GLOBAL_PREFS_ID
    );

    let res = http_client()
        .get(&url)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .header("Accept", "application/vnd.pgrst.object+json")
        .send()
        .await
        .ok()?;

    if res.status().is_success() {
        if let Ok(json) = res.json::<serde_json::Value>().await {
            if let Some(prefs_obj) = json.get("prefs_json") {
                if let Ok(prefs) =
                    serde_json::from_value::<crate::api::config::UserPreferences>(prefs_obj.clone())
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
    let endpoint = env::var("SUPABASE_URL")?;
    let key = env::var("SUPABASE_SERVICE_ROLE_KEY")?;

    let url = format!("{}/rest/v1/preferences", endpoint);

    let payload_val = serde_json::to_value(prefs).unwrap();

    let res = http_client()
        .post(&url)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .header("Prefer", "resolution=merge-duplicates")
        .json(&json!({
            "id": GLOBAL_PREFS_ID,
            "prefs_json": payload_val
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Supabase save preferences failed: {}", res.status()).into());
    }

    Ok(())
}

pub async fn get_recent_telemetry(limit: usize) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let endpoint = env::var("SUPABASE_URL")?;
    let key = env::var("SUPABASE_SERVICE_ROLE_KEY")?;

    let url = format!("{}/rest/v1/telemetry", endpoint);

    let response = http_client()
        .get(&url)
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {}", key))
        .query(&[
            ("order", "created_at.desc"),
            ("limit", &limit.clamp(1, 100).to_string()),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Supabase telemetry request failed: {}", response.status()).into());
    }

    let array = response.json::<Vec<Value>>().await?;

    Ok(array
        .into_iter()
        .map(|document| {
            json!({
                "event": document["event_type"],
                "timestamp": document["created_at"],
                "data": document["event_data"]
            })
        })
        .collect())
}
