use axum::{routing::get, Json, Router};
use reqwest::Client;
use serde::Serialize;
use std::time::Instant;

use crate::api::config::current_preferences;

#[derive(Serialize)]
pub struct ProviderStatus {
    provider: &'static str,
    configured: bool,
    status: &'static str,
    latency_ms: Option<u64>,
    message: String,
}

pub fn router() -> Router {
    Router::new().route("/providers/status", get(provider_status))
}

async fn provider_status() -> Json<Vec<ProviderStatus>> {
    let prefs = current_preferences();
    let client = Client::new();

    let (torbox, real_debrid, gemini) = tokio::join!(
        test_torbox(&client, prefs.torbox_api_key),
        test_real_debrid(&client, prefs.real_debrid_api_key),
        test_gemini(&client, prefs.gemini_api_key),
    );

    Json(vec![torbox, real_debrid, gemini])
}

async fn test_torbox(client: &Client, api_key: String) -> ProviderStatus {
    test_bearer_provider(
        client,
        "TorBox",
        api_key,
        "https://api.torbox.app/v1/api/user/me",
    )
    .await
}

async fn test_real_debrid(client: &Client, api_key: String) -> ProviderStatus {
    test_bearer_provider(
        client,
        "Real Debrid",
        api_key,
        "https://api.real-debrid.com/rest/1.0/user",
    )
    .await
}

async fn test_gemini(client: &Client, api_key: String) -> ProviderStatus {
    if api_key.is_empty() {
        return not_configured("Gemini");
    }

    let started = Instant::now();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash?key={}",
        api_key
    );
    match client.get(url).send().await {
        Ok(response) if response.status().is_success() => ok("Gemini", started),
        Ok(response) => error("Gemini", started, format!("HTTP {}", response.status())),
        Err(err) => error("Gemini", started, err.to_string()),
    }
}

async fn test_bearer_provider(
    client: &Client,
    provider: &'static str,
    api_key: String,
    url: &'static str,
) -> ProviderStatus {
    if api_key.is_empty() {
        return not_configured(provider);
    }

    let started = Instant::now();
    match client.get(url).bearer_auth(api_key).send().await {
        Ok(response) if response.status().is_success() => ok(provider, started),
        Ok(response) => error(provider, started, format!("HTTP {}", response.status())),
        Err(err) => error(provider, started, err.to_string()),
    }
}

fn not_configured(provider: &'static str) -> ProviderStatus {
    ProviderStatus {
        provider,
        configured: false,
        status: "not_configured",
        latency_ms: None,
        message: "No API key configured.".to_string(),
    }
}

fn ok(provider: &'static str, started: Instant) -> ProviderStatus {
    ProviderStatus {
        provider,
        configured: true,
        status: "ok",
        latency_ms: Some(started.elapsed().as_millis() as u64),
        message: "Connection verified.".to_string(),
    }
}

fn error(provider: &'static str, started: Instant, message: String) -> ProviderStatus {
    ProviderStatus {
        provider,
        configured: true,
        status: "error",
        latency_ms: Some(started.elapsed().as_millis() as u64),
        message,
    }
}
