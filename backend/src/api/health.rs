use axum::{routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    environment: String,
}

#[derive(Serialize)]
struct RootResponse {
    service: &'static str,
    status: &'static str,
    environment: String,
    health: &'static str,
    demo_manifest: &'static str,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
}

async fn root() -> Json<RootResponse> {
    Json(RootResponse {
        service: "atlas-backend",
        status: "ok",
        environment: current_environment(),
        health: "/health",
        demo_manifest: "/stremio/demo-install-token/manifest.json",
    })
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "atlas-backend",
        version: env!("CARGO_PKG_VERSION"),
        environment: current_environment(),
    })
}

fn current_environment() -> String {
    std::env::var("ATLAS_ENV").unwrap_or_else(|_| "local".to_string())
}
