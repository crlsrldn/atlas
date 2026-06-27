use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub fn log_event(event_name: &str, payload: Value) {
    let event_name = event_name.to_string();
    let mut payload = payload;
    crate::engines::privacy::redact_json(&mut payload);
    let payload_str = serde_json::to_string(&json!({
        "event": event_name,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": payload
    }))
    .unwrap_or_default();

    tokio::spawn(async move {
        if let (Ok(endpoint), Ok(project), Ok(key)) = (
            env::var("APPWRITE_ENDPOINT"),
            env::var("APPWRITE_PROJECT_ID"),
            env::var("APPWRITE_API_KEY"),
        ) {
            let url = format!(
                "{}/databases/atlas/collections/telemetry/documents",
                endpoint
            );

            let res = HTTP_CLIENT
                .post(&url)
                .header("X-Appwrite-Project", project)
                .header("X-Appwrite-Key", key)
                .json(&json!({
                    "documentId": "unique()",
                    "data": {
                        "telemetry_json": payload_str
                    }
                }))
                .send()
                .await;

            if let Err(e) = res {
                tracing::error!("Failed to log telemetry: {}", e);
            }
        }
    });
}
