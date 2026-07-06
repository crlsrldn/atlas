use crate::api::config::UserPreferences;
use crate::engines::identity::AtlasID;
use axum::{extract::Path, response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct ResolveRequest {
    pub stremio_id: String,
    pub install_token: Option<String>,
    pub prefs: UserPreferences,
    pub user_agent: Option<String>,
    #[serde(default)]
    pub monetization_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct ResolveHashRequest {
    pub prefs: UserPreferences,
    pub user_agent: Option<String>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    #[serde(default)]
    pub monetization_enabled: bool,
    #[serde(default)]
    pub cached: bool,
    pub install_token: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/internal/resolve", post(resolve))
        .route("/internal/resolve_hash/:provider/:hash", post(resolve_hash))
}

async fn resolve(Json(req): Json<ResolveRequest>) -> Json<Value> {
    let start_time = std::time::Instant::now();
    let mut stremio_id = req.stremio_id;
    if stremio_id.ends_with(".json") {
        stremio_id = stremio_id.trim_end_matches(".json").to_string();
    }

    let atlas_id = match AtlasID::from_stremio_id(&stremio_id) {
        Some(id) => id,
        None => return Json(json!({ "streams": [] })),
    };

    let token = req.install_token.as_deref().unwrap_or("demo");

    tracing::info!(
        "Resolving for stremio_id: {}, token: {}, prefs: {:?}",
        stremio_id,
        token,
        req.prefs
    );

    let smart_prefs = match req.user_agent.as_deref() {
        Some(ua) => crate::engines::ai_decision::infer_capabilities(ua, req.prefs),
        None => req.prefs,
    };

    let streams = crate::engines::playback::resolve_stream_for_tenant(
        atlas_id,
        smart_prefs,
        req.monetization_enabled,
        "global",
        token,
    )
    .await;

    tracing::info!("Resolved {} streams", streams.len());

    let mut res_4k = 0;
    let mut res_1080p = 0;
    let mut res_720p = 0;
    let mut res_unknown = 0;

    for stream in &streams {
        if let Some(desc) = &stream.description {
            if desc.contains("4K") || desc.contains("2160p") {
                res_4k += 1;
            } else if desc.contains("1080p") {
                res_1080p += 1;
            } else if desc.contains("720p") {
                res_720p += 1;
            } else {
                res_unknown += 1;
            }
        } else {
            res_unknown += 1;
        }
    }

    let latency_ms = start_time.elapsed().as_millis();

    crate::engines::telemetry::log_event(
        "streams_requested",
        serde_json::json!({
            "stremio_id": stremio_id,
            "streams_count": streams.len(),
            "latency_ms": latency_ms as u64,
            "install_token": token,
            "resolution_distribution": {
                "4k": res_4k,
                "1080p": res_1080p,
                "720p": res_720p,
                "unknown": res_unknown
            }
        }),
    );

    Json(json!({ "streams": streams }))
}

async fn resolve_hash(
    Path((provider, hash)): Path<(String, String)>,
    Json(req): Json<ResolveHashRequest>,
) -> axum::response::Response {
    let smart_prefs = match req.user_agent.as_deref() {
        Some(ua) => crate::engines::ai_decision::infer_capabilities(ua, req.prefs),
        None => req.prefs,
    };

    match provider.as_str() {
        "torbox" => {
            crate::api::resolve::resolve_torbox_with_key(
                hash,
                smart_prefs.torbox_api_key,
                None,
                req.user_agent.as_deref(),
                req.season,
                req.episode,
                req.cached,
                req.install_token,
            )
            .await
        }

        _ => (
            axum::http::StatusCode::FOUND,
            [("Location", "https://github.com/cindral/atlas")],
        )
            .into_response(),
    }
}
