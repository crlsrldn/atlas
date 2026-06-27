use axum::{http::HeaderMap, routing::get, Json, Router};
use serde::Serialize;

use crate::api::config::current_preferences;
use crate::engines::sources::{
    real_debrid::RealDebridProvider, torbox::TorBoxProvider, ProviderHealth, ProviderHealthStatus,
    SourceProvider,
};

#[derive(Serialize)]
pub struct ProviderStatus {
    provider: String,
    configured: bool,
    status: &'static str,
    latency_ms: Option<u64>,
    message: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/providers/status", get(provider_status))
        .route("/v1/providers/status", get(provider_status_for_tenant))
}

async fn provider_status() -> Json<Vec<ProviderStatus>> {
    let prefs = current_preferences();
    Json(provider_status_for_preferences(prefs).await)
}

async fn provider_status_for_tenant(headers: HeaderMap) -> Json<Vec<ProviderStatus>> {
    let tenant = crate::api::tenants::current_tenant(&headers);
    Json(provider_status_for_preferences(tenant.hydrated_preferences()).await)
}

async fn provider_status_for_preferences(
    prefs: crate::api::config::UserPreferences,
) -> Vec<ProviderStatus> {
    let torbox = TorBoxProvider {
        api_key: prefs.torbox_api_key,
    };
    let real_debrid = RealDebridProvider {
        api_key: prefs.real_debrid_api_key,
    };

    let (torbox_status, real_debrid_status, gemini_status) = tokio::join!(
        torbox.health(),
        real_debrid.health(),
        test_gemini(prefs.gemini_api_key),
    );

    vec![
        torbox_status.into(),
        real_debrid_status.into(),
        gemini_status,
    ]
}

async fn test_gemini(api_key: String) -> ProviderStatus {
    if api_key.is_empty() {
        return not_configured("Gemini");
    }

    let started = std::time::Instant::now();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash?key={}",
        api_key
    );
    match reqwest::Client::new().get(url).send().await {
        Ok(response) if response.status().is_success() => ok("Gemini", started),
        Ok(response) => error("Gemini", started, format!("HTTP {}", response.status())),
        Err(err) => error("Gemini", started, err.to_string()),
    }
}

fn not_configured(provider: &'static str) -> ProviderStatus {
    ProviderStatus {
        provider: provider.to_string(),
        configured: false,
        status: "not_configured",
        latency_ms: None,
        message: "No API key configured.".to_string(),
    }
}

fn ok(provider: &'static str, started: std::time::Instant) -> ProviderStatus {
    ProviderStatus {
        provider: provider.to_string(),
        configured: true,
        status: "ok",
        latency_ms: Some(started.elapsed().as_millis() as u64),
        message: "Connection verified.".to_string(),
    }
}

fn error(provider: &'static str, started: std::time::Instant, message: String) -> ProviderStatus {
    ProviderStatus {
        provider: provider.to_string(),
        configured: true,
        status: "error",
        latency_ms: Some(started.elapsed().as_millis() as u64),
        message,
    }
}

impl From<ProviderHealth> for ProviderStatus {
    fn from(health: ProviderHealth) -> Self {
        let status = match health.status {
            ProviderHealthStatus::Ok => "ok",
            ProviderHealthStatus::NotConfigured => "not_configured",
            ProviderHealthStatus::Error => "error",
        };

        Self {
            provider: health.provider_name,
            configured: health.configured,
            status,
            latency_ms: health.latency_ms,
            message: health.message,
        }
    }
}
