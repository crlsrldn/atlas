use serde_json::{json, Value};
use std::env;

pub fn log_event(event_name: &str, payload: Value) {
    let event_name = event_name.to_string();
    let mut payload = payload;
    crate::engines::privacy::redact_json(&mut payload);

    tokio::spawn(async move {
        if let (Ok(endpoint), Ok(key)) = (
            env::var("SUPABASE_URL"),
            env::var("SUPABASE_SERVICE_ROLE_KEY"),
        ) {
            let url = format!("{}/rest/v1/telemetry", endpoint);

            let res = crate::engines::http::client()
                .post(&url)
                .header("apikey", &key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .json(&json!({
                    "event_type": event_name,
                    "event_data": payload
                }))
                .send()
                .await;

            if let Err(e) = res {
                tracing::error!("Failed to log telemetry: {}", e);
            }
        }
    });
}
