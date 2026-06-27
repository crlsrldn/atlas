use crate::engines::identity::AtlasID;
use crate::engines::playback::resolve_detailed_streams;
use axum::{extract::Path, routing::get, Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new()
        .route("/inspect/:type/:id.json", get(inspect))
        .route("/inspect/:type/:id/:extra.json", get(inspect_with_extra))
}

async fn inspect(Path((_t, id)): Path<(String, String)>) -> Json<Value> {
    handle_inspect_request(id).await
}

async fn inspect_with_extra(Path((_t, id, _extra)): Path<(String, String, String)>) -> Json<Value> {
    handle_inspect_request(id).await
}

async fn handle_inspect_request(mut stremio_id: String) -> Json<Value> {
    if stremio_id.ends_with(".json") {
        stremio_id = stremio_id.trim_end_matches(".json").to_string();
    }

    let Some(atlas_id) = AtlasID::from_stremio_id(&stremio_id) else {
        return Json(json!({ "streams": [] }));
    };

    Json(json!({
        "id": stremio_id,
        "streams": resolve_detailed_streams(atlas_id).await
    }))
}
