use crate::engines::identity::AtlasID;
use crate::engines::playback::resolve_stream;
use axum::{extract::Path, routing::get, Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new()
        .route("/manifest.json", get(manifest))
        .route("/catalog/:type/:id.json", get(catalog))
        .route("/stream/:type/:id.json", get(stream))
        .route("/stream/:type/:id/:extra.json", get(stream_with_extra))
}

async fn manifest() -> Json<Value> {
    Json(manifest_payload())
}

fn manifest_payload() -> Value {
    json!({
        "id": "com.cindrallabs.atlas",
        "version": "0.1.0",
        "name": "Project Atlas",
        "description": "The intelligence layer for your media.",
        "types": ["movie", "series"],
        "catalogs": [
            {
                "type": "movie",
                "id": "atlas-ai",
                "name": "Atlas AI Recommendations"
            }
        ],
        "resources": [
            "stream",
            "catalog"
        ],
        "idPrefixes": ["tt", "tmdb:"]
    })
}

async fn catalog(Path((_t, id)): Path<(String, String)>) -> Json<Value> {
    if id == "atlas-ai" || id == "atlas-ai.json" {
        let recommendations = crate::engines::ai::get_movie_recommendations().await;

        let metas: Vec<Value> = recommendations
            .into_iter()
            .map(|rec| {
                json!({
                    "id": rec.imdb_id,
                    "type": "movie",
                    "name": rec.title,
                    "description": rec.reason,
                    // "poster": "" // We could add a poster here if we fetched it
                })
            })
            .collect();

        return Json(json!({ "metas": metas }));
    }

    Json(json!({ "metas": [] }))
}

async fn stream(Path((_t, id)): Path<(String, String)>) -> Json<Value> {
    handle_stream_request(id).await
}

async fn stream_with_extra(Path((_t, id, _extra)): Path<(String, String, String)>) -> Json<Value> {
    // For series, id will be tt1234567:1:2 (IMDb ID:Season:Episode)
    // We treat it as one string for now and let the identity parser handle it.
    handle_stream_request(id).await
}

async fn handle_stream_request(mut stremio_id: String) -> Json<Value> {
    let start_time = std::time::Instant::now();
    tracing::info!("Received request for stream id: {}", stremio_id);

    if stremio_id.ends_with(".json") {
        stremio_id = stremio_id.trim_end_matches(".json").to_string();
    }

    let atlas_id = match AtlasID::from_stremio_id(&stremio_id) {
        Some(id) => id,
        None => return Json(json!({ "streams": [] })),
    };

    let streams = resolve_stream(atlas_id).await;
    let latency_ms = start_time.elapsed().as_millis() as u64;

    crate::engines::telemetry::log_event(
        "stream_resolved",
        json!({
            "stremio_id": stremio_id,
            "latency_ms": latency_ms,
            "streams_found": streams.len()
        }),
    );

    Json(json!({ "streams": streams }))
}

#[cfg(test)]
mod tests {
    use super::manifest_payload;

    #[test]
    fn manifest_exposes_stremio_addon_contract() {
        let manifest = manifest_payload();

        assert_eq!(manifest["id"], "com.cindrallabs.atlas");
        assert_eq!(manifest["resources"][0], "stream");
        assert_eq!(manifest["resources"][1], "catalog");
        assert!(manifest["types"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "movie"));
        assert!(manifest["types"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "series"));
        assert!(manifest["idPrefixes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "tt"));
    }
}
