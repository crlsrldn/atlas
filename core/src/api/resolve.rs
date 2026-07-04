use crate::api::config::current_preferences;
use crate::engines::history::{
    fallback_candidates, fallback_candidates_scope, media_key_from_hash, media_key_from_hash_scope,
    record_playback, record_playback_scope,
};
use axum::http::StatusCode;
use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use regex::Regex;
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
    Router::new().route("/resolve/torbox/:hash", get(resolve_torbox))
}

async fn resolve_torbox(Path(hash): Path<String>) -> axum::response::Response {
    let prefs = current_preferences();
    resolve_torbox_with_key(
        hash,
        prefs.torbox_api_key,
        None,
        None,
        None,
        None,
        false,
        None,
    )
    .await
}

pub async fn resolve_torbox_with_key(
    hash: String,
    api_key: String,
    history_scope: Option<&str>,
    user_agent: Option<&str>,
    season: Option<u32>,
    episode: Option<u32>,
    is_cached: bool,
    user_id: Option<String>,
) -> axum::response::Response {
    if api_key.is_empty() {
        return (StatusCode::FOUND, [("Location", "https://torbox.app")]).into_response();
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
                    let file_id = match select_best_video_file(&data.files, season, episode) {
                        Some(file_id) => file_id,
                        None => {
                            find_best_torbox_file_id(&client, &api_key, torrent_id, season, episode)
                                .await
                                .unwrap_or(1)
                        }
                    };
                    let dl_url = format!("https://api.torbox.app/v1/api/torrents/requestdl?token={}&torrent_id={}&file_id={}&redirect=true", api_key, torrent_id, file_id);

                    record_provider_playback(history_scope, "TorBox", &hash, true);
                    crate::engines::telemetry::log_event(
                        "playback_started",
                        serde_json::json!({
                            "provider": "torbox",
                            "success": true,
                            "user_agent": user_agent,
                            "user_id": user_id.clone()
                        }),
                    );

                    // Feature 5: Automated Background Caching
                    if !is_cached {
                        // The torrent was just added to TorBox and is downloading.
                        // We redirect the user to a static placeholder video so Stremio doesn't hang.
                        let placeholder_url = "https://www.w3schools.com/html/mov_bbb.mp4";
                        return (StatusCode::FOUND, [("Location", placeholder_url)])
                            .into_response();
                    }

                    return (StatusCode::FOUND, [("Location", dl_url)]).into_response();
                }
            }
        }
    }

    record_provider_playback(history_scope, "TorBox", &hash, false);
    crate::engines::telemetry::log_event(
        "playback_started",
        serde_json::json!({
            "provider": "torbox",
            "success": false,
            "user_agent": user_agent,
            "user_id": user_id
        }),
    );

    fallback_redirect_for_hash(history_scope, &hash, "https://torbox.app")
}

async fn find_best_torbox_file_id(
    client: &reqwest::Client,
    api_key: &str,
    torrent_id: u64,
    season: Option<u32>,
    episode: Option<u32>,
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
        if let Some(file_id) = select_best_video_file_from_json(&json, season, episode) {
            return Some(file_id);
        }
    }

    None
}

fn select_best_video_file(
    files: &[TorboxFile],
    season: Option<u32>,
    episode: Option<u32>,
) -> Option<u64> {
    let candidates = files
        .iter()
        .filter(|file| is_video_name(file.name.as_deref().unwrap_or_default()))
        .filter_map(|file| {
            let id = file.file_id.or(file.id)?;
            let size = file.size_bytes.or(file.size).unwrap_or(0);
            let name = file.name.clone().unwrap_or_default();
            Some((id, size, name))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return None;
    }

    if let (Some(s), Some(e)) = (season, episode) {
        let matched = candidates
            .iter()
            .filter(|(_, _, name)| episode_marker_matches(name, s, e))
            .collect::<Vec<_>>();
        if !matched.is_empty() {
            return matched
                .into_iter()
                .max_by_key(|(_, size, _)| *size)
                .map(|(id, _, _)| *id);
        }
    }

    candidates
        .into_iter()
        .max_by_key(|(_, size, _)| *size)
        .map(|(id, _, _)| id)
}

fn select_best_video_file_from_json(
    value: &Value,
    season: Option<u32>,
    episode: Option<u32>,
) -> Option<u64> {
    let mut candidates = Vec::new();
    collect_torbox_file_candidates(value, &mut candidates);

    if candidates.is_empty() {
        return None;
    }

    if let (Some(s), Some(e)) = (season, episode) {
        let matched = candidates
            .iter()
            .filter(|(_, _, name)| episode_marker_matches(name, s, e))
            .collect::<Vec<_>>();
        if !matched.is_empty() {
            return matched
                .into_iter()
                .max_by_key(|(_, size, _)| *size)
                .map(|(id, _, _)| *id);
        }
    }

    candidates
        .into_iter()
        .max_by_key(|(_, size, _)| *size)
        .map(|(id, _, _)| id)
}

fn episode_marker_matches(evidence: &str, season: u32, episode: u32) -> bool {
    let evidence_lower = evidence.to_lowercase();

    // 1. Direct standard matches
    let direct_pattern = format!(
        r"(?ix)(
            s0*{season}\s*[ex]\s*0*{episode}\b |
            \b0*{season}\s*x\s*0*{episode}\b |
            season\s*0*{season}\s*episode\s*0*{episode}\b
        )"
    );
    if let Ok(re) = Regex::new(&direct_pattern) {
        if re.is_match(&evidence_lower) {
            return true;
        }
    }

    // 2. Anime fallback for Season 1
    if season == 1 {
        let anime_pattern = format!(
            r"(?ix)(
                \b(?:e|ep|episode)\s*0*{episode}\b |
                \s+-\s+0*{episode}\b
            )"
        );
        if let Ok(anime_re) = Regex::new(&anime_pattern) {
            if anime_re.is_match(&evidence_lower) {
                return true;
            }
        }
    }

    // 3. Multi-episode range matches (e.g. S01E01-E03)
    let range_pattern =
        format!(r"(?ix)s0*{season}\s*e\s*(?P<start>\d{{1,3}})\s*-\s*(?:e\s*)?(?P<end>\d{{1,3}})\b");
    if let Ok(re) = Regex::new(&range_pattern) {
        for cap in re.captures_iter(&evidence_lower) {
            if let (Some(start), Some(end)) = (cap.name("start"), cap.name("end")) {
                if let (Ok(start_num), Ok(end_num)) =
                    (start.as_str().parse::<u32>(), end.as_str().parse::<u32>())
                {
                    if start_num <= end_num && episode >= start_num && episode <= end_num {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn collect_torbox_file_candidates(value: &Value, candidates: &mut Vec<(u64, u64, String)>) {
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
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            if is_video_name(name) {
                if let Some(id) = map
                    .get("file_id")
                    .or_else(|| map.get("id"))
                    .and_then(|v| v.as_u64())
                {
                    let size = map
                        .get("size_bytes")
                        .or_else(|| map.get("size"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    candidates.push((id, size, name.to_string()));
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

fn fallback_redirect_for_hash(
    history_scope: Option<&str>,
    hash: &str,
    provider_home: &'static str,
) -> axum::response::Response {
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
            return (StatusCode::FOUND, [("Location", candidate.url.clone())]).into_response();
        }
    }

    (StatusCode::FOUND, [("Location", provider_home)]).into_response()
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_episode_marker_matches() {
        use super::episode_marker_matches;

        assert!(episode_marker_matches("Show S01E02 1080p", 1, 2));
        assert!(episode_marker_matches("Show 1x02 1080p", 1, 2));
        assert!(episode_marker_matches(
            "Show Season 1 Episode 2 1080p",
            1,
            2
        ));
        assert!(episode_marker_matches("Show S01E01-E02", 1, 2));
        assert!(episode_marker_matches("Show S01E01-E03", 1, 2));
        assert!(episode_marker_matches("Show S01E02-E03", 1, 2));

        // Anime
        assert!(episode_marker_matches("Anime E13 1080p", 1, 13));
        assert!(episode_marker_matches("Anime - 13 1080p", 1, 13));
        assert!(episode_marker_matches("Anime ep13 1080p", 1, 13));
        assert!(episode_marker_matches("Anime EP 13 1080p", 1, 13));

        // Fails
        assert!(!episode_marker_matches("Show S01E02 1080p", 1, 3));
        assert!(!episode_marker_matches("Show S01E020 1080p", 1, 2)); // word boundary
        assert!(!episode_marker_matches("Anime E13 1080p", 2, 13)); // Anime logic only triggers for season 1
    }
}
