use crate::api::tenants::{charge_resolve, tenant_by_install_token};
use crate::engines::cache::{get_json, scoped_key, set_json, SOURCE_RESULTS_TTL};
use crate::engines::identity::AtlasID;
use crate::engines::playback::{
    resolve_detailed_streams_with_preferences, resolve_stream_for_tenant,
};
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new()
        .route("/stremio/:install_token/manifest.json", get(manifest))
        .route("/stremio/:install_token/stream/:type/:id.json", get(stream))
        .route(
            "/stremio/:install_token/stream/:type/:id/:extra.json",
            get(stream_with_extra),
        )
        .route(
            "/stremio/:install_token/inspect/:type/:id.json",
            get(inspect),
        )
        .route(
            "/stremio/:install_token/inspect/:type/:id/:extra.json",
            get(inspect_with_extra),
        )
        .route(
            "/stremio/:install_token/resolve/:provider/:hash",
            get(resolve_hosted),
        )
}

async fn manifest(Path(install_token): Path<String>) -> Response {
    if tenant_by_install_token(&install_token).is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "invalid_install_token" })),
        )
            .into_response();
    }

    Json(json!({
        "id": format!("com.cindrallabs.atlas.{}", install_token),
        "version": env!("CARGO_PKG_VERSION"),
        "name": "Atlas Cloud",
        "description": "Tenant-scoped Atlas Smart Play resolver.",
        "types": ["movie", "series"],
        "catalogs": [
            {
                "type": "movie",
                "id": "atlas-ai",
                "name": "Atlas AI Recommendations"
            }
        ],
        "resources": ["stream", "catalog"],
        "idPrefixes": ["tt", "tmdb:"]
    }))
    .into_response()
}

async fn stream(
    Path((install_token, _media_type, id)): Path<(String, String, String)>,
) -> Response {
    handle_stream_request(install_token, id).await
}

async fn stream_with_extra(
    Path((install_token, _media_type, id, _extra)): Path<(String, String, String, String)>,
) -> Response {
    handle_stream_request(install_token, id).await
}

async fn handle_stream_request(install_token: String, mut stremio_id: String) -> Response {
    let Ok(tenant) = charge_resolve(&install_token) else {
        return (StatusCode::FORBIDDEN, Json(json!({ "streams": [] }))).into_response();
    };

    if stremio_id.ends_with(".json") {
        stremio_id = stremio_id.trim_end_matches(".json").to_string();
    }

    let cache_key = scoped_key(&tenant.user_id, "stremio_streams", &stremio_id);
    if let Some(cached) = get_json(&cache_key) {
        return Json(cached).into_response();
    }

    let Some(atlas_id) = AtlasID::from_stremio_id(&stremio_id) else {
        return Json(json!({ "streams": [] })).into_response();
    };

    let streams = resolve_stream_for_tenant(
        atlas_id,
        tenant.hydrated_preferences(),
        &tenant.user_id,
        &install_token,
    )
    .await;
    let payload = json!({ "streams": streams });
    set_json(cache_key, payload.clone(), SOURCE_RESULTS_TTL);

    Json(payload).into_response()
}

async fn inspect(
    Path((install_token, _media_type, id)): Path<(String, String, String)>,
) -> Response {
    handle_inspect_request(install_token, id).await
}

async fn inspect_with_extra(
    Path((install_token, _media_type, id, _extra)): Path<(String, String, String, String)>,
) -> Response {
    handle_inspect_request(install_token, id).await
}

async fn handle_inspect_request(install_token: String, mut stremio_id: String) -> Response {
    let Some(tenant) = tenant_by_install_token(&install_token) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "invalid_install_token" })),
        )
            .into_response();
    };

    if stremio_id.ends_with(".json") {
        stremio_id = stremio_id.trim_end_matches(".json").to_string();
    }

    let Some(atlas_id) = AtlasID::from_stremio_id(&stremio_id) else {
        return Json(json!({ "streams": [] })).into_response();
    };

    let streams = resolve_detailed_streams_with_preferences(
        atlas_id,
        tenant.hydrated_preferences(),
        &tenant.user_id,
        Some(&install_token),
    )
    .await;

    Json(json!({
        "id": stremio_id,
        "user_id": tenant.user_id,
        "streams": streams
    }))
    .into_response()
}

async fn resolve_hosted(
    Path((install_token, provider, hash)): Path<(String, String, String)>,
) -> Response {
    let Some(tenant) = tenant_by_install_token(&install_token) else {
        return Redirect::temporary("https://cindrallabs.com").into_response();
    };

    let prefs = tenant.hydrated_preferences();
    let redirect = match provider.as_str() {
        "torbox" => {
            crate::api::resolve::resolve_torbox_with_key(
                hash,
                prefs.torbox_api_key,
                Some(tenant.user_id.as_str()),
            )
            .await
        }
        "realdebrid" => {
            crate::api::resolve::resolve_realdebrid_with_key(
                hash,
                prefs.real_debrid_api_key,
                Some(tenant.user_id.as_str()),
            )
            .await
        }
        _ => Redirect::temporary("https://cindrallabs.com"),
    };

    redirect.into_response()
}

#[allow(dead_code)]
fn _cache_policy_marker(_: Value) {
    // Cache resolver decisions and source metadata only. Never cache media bytes.
}
