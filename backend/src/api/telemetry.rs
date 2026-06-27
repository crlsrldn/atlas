use axum::{routing::get, Json, Router};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct TelemetryResponse {
    events: Vec<Value>,
    source: &'static str,
    message: Option<String>,
}

pub fn router() -> Router {
    Router::new().route("/telemetry/recent", get(recent_telemetry))
}

async fn recent_telemetry() -> Json<TelemetryResponse> {
    match crate::api::cloud::get_recent_telemetry(50).await {
        Ok(events) => Json(TelemetryResponse {
            events,
            source: "appwrite",
            message: None,
        }),
        Err(err) => Json(TelemetryResponse {
            events: Vec::new(),
            source: "backend",
            message: Some(format!("Telemetry unavailable: {}", err)),
        }),
    }
}
