pub mod api;
pub mod engines;

use axum::{
    http::{HeaderValue, Method},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load .env
    let _ = dotenvy::dotenv();

    // Initialize Cloud Preferences
    api::config::init_preferences().await;

    let allowed_origins = [
        HeaderValue::from_static("http://localhost:1420"),
        HeaderValue::from_static("http://127.0.0.1:1420"),
        HeaderValue::from_static("http://localhost:5173"),
        HeaderValue::from_static("http://127.0.0.1:5173"),
        HeaderValue::from_static("tauri://localhost"),
    ];

    let local_app_cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_headers(Any);

    let stremio_cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::OPTIONS])
        .allow_origin(Any)
        .allow_headers(Any);

    // Build our application with routes
    let app = Router::new()
        .nest("/", api::stremio::router().layer(stremio_cors.clone()))
        .nest("/", api::resolve::router().layer(stremio_cors))
        .nest("/", api::inspect::router().layer(local_app_cors.clone()))
        .nest("/", api::config::router().layer(local_app_cors.clone()))
        .nest("/", api::health::router().layer(local_app_cors.clone()))
        .nest("/", api::providers::router().layer(local_app_cors.clone()))
        .nest("/", api::telemetry::router().layer(local_app_cors));

    // Run it
    let addr = std::env::var("ATLAS_BIND_ADDR")
        .ok()
        .and_then(|value| value.parse::<SocketAddr>().ok())
        .unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 3000)));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
