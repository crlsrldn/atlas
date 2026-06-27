use crate::api::config::current_preferences;
use crate::engines::history::{
    fallback_candidates, fallback_candidates_scope, media_key_from_hash, media_key_from_hash_scope,
    record_playback, record_playback_scope,
};
use axum::{extract::Path, response::Redirect, routing::get, Router};
use reqwest;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct TorboxCreateResponse {
    success: bool,
    data: Option<TorboxTorrentData>,
}

#[derive(Deserialize)]
struct TorboxTorrentData {
    torrent_id: u64,
    #[serde(default)]
    files: Vec<TorboxFile>,
}

#[derive(Clone, Deserialize)]
struct TorboxFile {
    id: Option<u64>,
    file_id: Option<u64>,
    name: Option<String>,
    size: Option<u64>,
    size_bytes: Option<u64>,
}

pub fn router() -> Router {
    Router::new()
        .route("/resolve/torbox/:hash", get(resolve_torbox))
        .route("/resolve/realdebrid/:hash", get(resolve_realdebrid))
}

async fn resolve_torbox(Path(hash): Path<String>) -> Redirect {
    let prefs = current_preferences();
    resolve_torbox_with_key(hash, prefs.torbox_api_key, None).await
}

pub async fn resolve_torbox_with_key(
    hash: String,
    api_key: String,
    history_scope: Option<&str>,
) -> Redirect {
    if api_key.is_empty() {
        return Redirect::temporary("https://torbox.app");
    }

    let client = reqwest::Client::new();
    let magnet = format!("magnet:?xt=urn:btih:{}", hash);

    // 1. Create/Add the torrent
    let create_res = client
        .post("https://api.torbox.app/v1/api/torrents/createtorrent")
        .bearer_auth(&api_key)
        .form(&[("magnet", magnet.as_str())])
        .send()
        .await;

    if let Ok(res) = create_res {
        if let Ok(json) = res.json::<TorboxCreateResponse>().await {
            if json.success {
                if let Some(data) = json.data {
                    let torrent_id = data.torrent_id;
                    let file_id = match select_largest_video_file(&data.files) {
                        Some(file_id) => file_id,
                        None => find_largest_torbox_file_id(&client, &api_key, torrent_id)
                            .await
                            .unwrap_or(1),
                    };
                    let dl_url = format!("https://api.torbox.app/v1/api/torrents/requestdl?token={}&torrent_id={}&file_id={}&redirect=true", api_key, torrent_id, file_id);

                    record_provider_playback(history_scope, "TorBox", &hash, true);
                    crate::engines::telemetry::log_event(
                        "playback_started",
                        serde_json::json!({
                            "provider": "torbox",
                            "success": true
                        }),
                    );

                    return Redirect::temporary(&dl_url);
                }
            }
        }
    }

    record_provider_playback(history_scope, "TorBox", &hash, false);
    crate::engines::telemetry::log_event(
        "playback_started",
        serde_json::json!({
            "provider": "torbox",
            "success": false
        }),
    );

    fallback_redirect_for_hash(history_scope, &hash, "https://torbox.app")
}

async fn find_largest_torbox_file_id(
    client: &reqwest::Client,
    api_key: &str,
    torrent_id: u64,
) -> Option<u64> {
    let candidates = [
        format!(
            "https://api.torbox.app/v1/api/torrents/mylist?id={}",
            torrent_id
        ),
        format!(
            "https://api.torbox.app/v1/api/torrents/mylist?torrent_id={}",
            torrent_id
        ),
    ];

    for url in candidates {
        let Ok(response) = client.get(url).bearer_auth(api_key).send().await else {
            continue;
        };
        let Ok(json) = response.json::<Value>().await else {
            continue;
        };
        if let Some(file_id) = select_largest_video_file_from_json(&json) {
            return Some(file_id);
        }
    }

    None
}

fn select_largest_video_file(files: &[TorboxFile]) -> Option<u64> {
    files
        .iter()
        .filter(|file| is_video_name(file.name.as_deref().unwrap_or_default()))
        .filter_map(|file| {
            let id = file.file_id.or(file.id)?;
            let size = file.size_bytes.or(file.size).unwrap_or(0);
            Some((id, size))
        })
        .max_by_key(|(_, size)| *size)
        .map(|(id, _)| id)
}

fn select_largest_video_file_from_json(value: &Value) -> Option<u64> {
    let mut candidates = Vec::new();
    collect_torbox_file_candidates(value, &mut candidates);
    candidates
        .into_iter()
        .max_by_key(|(_, size)| *size)
        .map(|(id, _)| id)
}

fn collect_torbox_file_candidates(value: &Value, candidates: &mut Vec<(u64, u64)>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_torbox_file_candidates(item, candidates);
            }
        }
        Value::Object(map) => {
            let name = map
                .get("name")
                .or_else(|| map.get("filename"))
                .or_else(|| map.get("path"))
                .and_then(Value::as_str)
                .unwrap_or_default();

            let id = map
                .get("file_id")
                .or_else(|| map.get("id"))
                .and_then(Value::as_u64);

            if let Some(id) = id {
                if is_video_name(name) {
                    let size = map
                        .get("size_bytes")
                        .or_else(|| map.get("size"))
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    candidates.push((id, size));
                }
            }

            for child in map.values() {
                collect_torbox_file_candidates(child, candidates);
            }
        }
        _ => {}
    }
}

fn is_video_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    [".mkv", ".mp4", ".avi", ".mov", ".m4v", ".webm"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

#[derive(Deserialize)]
struct RDAddMagnetResponse {
    id: String,
}

#[derive(Deserialize)]
struct RDInfoResponse {
    #[serde(default)]
    files: Vec<RDFile>,
    #[serde(default)]
    links: Vec<String>,
}

#[derive(Clone, Deserialize)]
struct RDFile {
    id: u64,
    path: String,
    #[serde(default)]
    bytes: u64,
    #[serde(default)]
    selected: u8,
}

#[derive(Deserialize)]
struct RDUnrestrictResponse {
    download: String,
}

async fn resolve_realdebrid(Path(hash): Path<String>) -> Redirect {
    let prefs = current_preferences();
    resolve_realdebrid_with_key(hash, prefs.real_debrid_api_key, None).await
}

pub async fn resolve_realdebrid_with_key(
    hash: String,
    api_key: String,
    history_scope: Option<&str>,
) -> Redirect {
    if api_key.is_empty() {
        return Redirect::temporary("https://real-debrid.com");
    }

    let client = reqwest::Client::new();
    let magnet = format!("magnet:?xt=urn:btih:{}", hash);

    // 1. Add Magnet
    let add_res = client
        .post("https://api.real-debrid.com/rest/1.0/torrents/addMagnet")
        .bearer_auth(&api_key)
        .form(&[("magnet", magnet.as_str())])
        .send()
        .await;

    if let Ok(res) = add_res {
        if let Ok(json) = res.json::<RDAddMagnetResponse>().await {
            let id = json.id;

            if let Some(download) = resolve_real_debrid_download(&client, &api_key, &id).await {
                record_provider_playback(history_scope, "Real Debrid", &hash, true);
                crate::engines::telemetry::log_event(
                    "playback_started",
                    serde_json::json!({
                        "provider": "real_debrid",
                        "success": true
                    }),
                );
                return Redirect::temporary(&download);
            }
        }
    }

    record_provider_playback(history_scope, "Real Debrid", &hash, false);
    crate::engines::telemetry::log_event(
        "playback_started",
        serde_json::json!({
            "provider": "real_debrid",
            "success": false
        }),
    );

    fallback_redirect_for_hash(history_scope, &hash, "https://real-debrid.com")
}

fn fallback_redirect_for_hash(
    history_scope: Option<&str>,
    hash: &str,
    provider_home: &'static str,
) -> Redirect {
    let media_key = match history_scope {
        Some(scope) => media_key_from_hash_scope(scope, hash),
        None => media_key_from_hash(hash),
    };

    if let Some(media_key) = media_key {
        let failed_hash_fragment = hash.to_lowercase();
        let candidates = match history_scope {
            Some(scope) => fallback_candidates_scope(scope, &media_key, ""),
            None => fallback_candidates(&media_key, ""),
        };

        if let Some(candidate) = candidates
            .into_iter()
            .find(|candidate| !candidate.hash.eq_ignore_ascii_case(&failed_hash_fragment))
        {
            return Redirect::temporary(&candidate.url);
        }
    }

    Redirect::temporary(provider_home)
}

fn record_provider_playback(
    history_scope: Option<&str>,
    provider: &str,
    hash: &str,
    success: bool,
) {
    match history_scope {
        Some(scope) => record_playback_scope(scope, provider, hash, success),
        None => record_playback(provider, hash, success),
    }
}

async fn resolve_real_debrid_download(
    client: &reqwest::Client,
    api_key: &str,
    torrent_id: &str,
) -> Option<String> {
    let info_url = format!(
        "https://api.real-debrid.com/rest/1.0/torrents/info/{}",
        torrent_id
    );

    let info = client
        .get(&info_url)
        .bearer_auth(api_key)
        .send()
        .await
        .ok()?
        .json::<RDInfoResponse>()
        .await
        .ok()?;

    let file_id = select_largest_real_debrid_video_file(&info.files).or_else(|| {
        info.files
            .iter()
            .find(|file| file.selected == 1)
            .map(|file| file.id)
    })?;

    let select_res = client
        .post(format!(
            "https://api.real-debrid.com/rest/1.0/torrents/selectFiles/{}",
            torrent_id
        ))
        .bearer_auth(api_key)
        .form(&[("files", file_id.to_string())])
        .send()
        .await
        .ok()?;

    if !select_res.status().is_success() {
        tracing::warn!(status = %select_res.status(), "real debrid selectFiles failed");
        return None;
    }

    let info = client
        .get(info_url)
        .bearer_auth(api_key)
        .send()
        .await
        .ok()?
        .json::<RDInfoResponse>()
        .await
        .ok()?;

    let link = info.links.first()?;

    client
        .post("https://api.real-debrid.com/rest/1.0/unrestrict/link")
        .bearer_auth(api_key)
        .form(&[("link", link.as_str())])
        .send()
        .await
        .ok()?
        .json::<RDUnrestrictResponse>()
        .await
        .ok()
        .map(|response| response.download)
}

fn select_largest_real_debrid_video_file(files: &[RDFile]) -> Option<u64> {
    files
        .iter()
        .filter(|file| is_video_name(&file.path))
        .max_by_key(|file| file.bytes)
        .map(|file| file.id)
}

#[cfg(test)]
mod tests {
    use super::{
        select_largest_real_debrid_video_file, select_largest_video_file_from_json, RDFile,
    };
    use serde_json::json;

    #[test]
    fn selects_largest_video_file_from_nested_torbox_payload() {
        let payload = json!({
            "data": {
                "files": [
                    { "id": 1, "name": "sample.mkv", "size": 100 },
                    { "id": 2, "name": "movie.mkv", "size": 10_000 },
                    { "id": 3, "name": "poster.jpg", "size": 20_000 }
                ]
            }
        });

        assert_eq!(select_largest_video_file_from_json(&payload), Some(2));
    }

    #[test]
    fn selects_largest_real_debrid_video_file() {
        let files = vec![
            RDFile {
                id: 1,
                path: "/poster.jpg".to_string(),
                bytes: 20_000,
                selected: 0,
            },
            RDFile {
                id: 2,
                path: "/sample.mkv".to_string(),
                bytes: 100,
                selected: 0,
            },
            RDFile {
                id: 3,
                path: "/movie.mkv".to_string(),
                bytes: 10_000,
                selected: 0,
            },
        ];

        assert_eq!(select_largest_real_debrid_video_file(&files), Some(3));
    }
}
