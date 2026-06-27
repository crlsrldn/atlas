pub mod api;
pub mod engines;

use axum::{
    http::Method,
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(false)
        .init();

    // Load .env
    let _ = dotenvy::dotenv();

    // Initialize Cloud Preferences
    api::config::init_preferences().await;

    let internal_cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(Any)
        .allow_headers(Any);

    // Build our application with routes
    let app = Router::new()
        .nest("/", api::internal::router().layer(internal_cors.clone()))
        .nest("/", api::inspect::router().layer(internal_cors.clone()))
        .nest("/", api::config::router().layer(internal_cors.clone()))
        .nest("/", api::health::router().layer(internal_cors.clone()))
        .nest("/", api::providers::router().layer(internal_cors.clone()))
        .nest("/", api::telemetry::router().layer(internal_cors));

    // Run it
    let addr = std::env::var("ATLAS_BIND_ADDR")
        .ok()
        .and_then(|value| value.parse::<SocketAddr>().ok())
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000)));
    tracing::info!(bind_addr = %addr, "backend listening");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
