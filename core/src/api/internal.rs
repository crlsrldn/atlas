use crate::api::config::UserPreferences;
use crate::engines::identity::AtlasID;
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct ResolveRequest {
    pub stremio_id: String,
    pub install_token: Option<String>,
    pub prefs: UserPreferences,
}

pub fn router() -> Router {
    Router::new()
        .route("/internal/resolve", post(resolve))
        .route("/internal/resolve_hash/:provider/:hash", get(resolve_hash))
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

async fn resolve_hash(Path((provider, hash)): Path<(String, String)>) -> axum::response::Redirect {
    crate::api::resolve::handle_resolve_redirect(&provider, &hash).await
}
