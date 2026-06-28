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
}

#[derive(Debug, Deserialize)]
pub struct ResolveHashRequest {
    pub prefs: UserPreferences,
}

pub fn router() -> Router {
    Router::new()
        .route("/internal/resolve", post(resolve))
        .route("/internal/resolve_hash/:provider/:hash", post(resolve_hash))
}

async fn resolve(Json(req): Json<ResolveRequest>) -> Json<Value> {
    let mut stremio_id = req.stremio_id;
    if stremio_id.ends_with(".json") {
        stremio_id = stremio_id.trim_end_matches(".json").to_string();
    }

    let atlas_id = match AtlasID::from_stremio_id(&stremio_id) {
        Some(id) => id,
        None => return Json(json!({ "streams": [] })),
    };

    let token = req.install_token.as_deref().unwrap_or("demo");

    let streams =
        crate::engines::playback::resolve_stream_for_tenant(atlas_id, req.prefs, "global", token)
            .await;

    Json(json!({ "streams": streams }))
}

async fn resolve_hash(
    Path((provider, hash)): Path<(String, String)>,
    Json(req): Json<ResolveHashRequest>,
) -> axum::response::Response {
    match provider.as_str() {
        "torbox" => {
            crate::api::resolve::resolve_torbox_with_key(hash, req.prefs.torbox_api_key, None).await
        }
        "realdebrid" => {
            crate::api::resolve::resolve_realdebrid_with_key(
                hash,
                req.prefs.real_debrid_api_key,
                None,
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
