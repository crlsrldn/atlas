use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    environment: String,
}

pub fn router() -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "atlas-backend",
        version: env!("CARGO_PKG_VERSION"),
        environment: std::env::var("ATLAS_ENV").unwrap_or_else(|_| "local".to_string()),
    })
}
