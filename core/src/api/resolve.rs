use crate::api::config::current_preferences;
use crate::engines::cache::{get_json, scoped_key, set_json, PLAYBACK_URL_TTL, TORRENT_HANDLE_TTL};
use crate::engines::history::{
    fallback_candidates, fallback_candidates_scope, media_key_from_hash, media_key_from_hash_scope,
    record_playback, record_playback_scope,
};
use crate::engines::http;
use axum::http::StatusCode;
use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

const LOCAL_SCOPE: &str = "local";

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

#[allow(clippy::too_many_arguments)]
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
        // Redirecting here would hand a video player torbox.app's HTML. Say
        // what actually happened instead.
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({
                "status": "not_configured",
                "message": "No TorBox API key is configured for this profile."
            })),
        )
            .into_response();
    }

    let scope = user_id
        .clone()
        .or_else(|| history_scope.map(str::to_string))
        .unwrap_or_else(|| LOCAL_SCOPE.to_string());
    // Season packs resolve to a different file per episode, so the cache key
    // must carry the episode context alongside the hash.
    let cache_id = playback_cache_id(&hash, season, episode);

    // Player reconnects (seeks, range re-requests) hit this endpoint repeatedly;
    // a cached CDN URL turns those into a single cheap redirect.
    if is_cached {
        if let Some(url) = cached_playback_url(&scope, &cache_id) {
            crate::engines::telemetry::log_event(
                "playback_started",
                serde_json::json!({
                    "provider": "torbox",
                    "success": true,
                    "cached": true,
                    "user_agent": user_agent,
                    "user_id": user_id
                }),
            );
            return (StatusCode::FOUND, [("Location", url)]).into_response();
        }
    }

    let client = http::client();

    // Feature 5: Automated Background Caching. The torrent is not cached on
    // TorBox yet, so requesting a download link would fail — just queue it so
    // TorBox starts downloading, and tell the client to retry shortly.
    if !is_cached {
        if add_torbox_magnet(client, &hash, &api_key).await.is_some() {
            record_provider_outcome(history_scope, "TorBox", &hash, ResolveOutcome::Queued);
            crate::engines::telemetry::log_event(
                "playback_queued",
                serde_json::json!({
                    "provider": "torbox",
                    "success": true,
                    "cached": false,
                    "user_agent": user_agent,
                    "user_id": user_id
                }),
            );
            return preparing_response();
        }
    } else if let Some(url) =
        resolve_torbox_playback(client, &hash, &api_key, &scope, &cache_id, season, episode).await
    {
        record_provider_outcome(history_scope, "TorBox", &hash, ResolveOutcome::Played);
        crate::engines::telemetry::log_event(
            "playback_started",
            serde_json::json!({
                "provider": "torbox",
                "success": true,
                "cached": false,
                "user_agent": user_agent,
                "user_id": user_id.clone()
            }),
        );
        return (StatusCode::FOUND, [("Location", url)]).into_response();
    }

    record_provider_outcome(history_scope, "TorBox", &hash, ResolveOutcome::Failed);
    crate::engines::telemetry::log_event(
        "playback_started",
        serde_json::json!({
            "provider": "torbox",
            "success": false,
            "user_agent": user_agent,
            "user_id": user_id,
            "error": "Failed to resolve playable link",
            "stremio_id": hash
        }),
    );

    fallback_redirect_for_hash(history_scope, &hash, "https://torbox.app")
}

/// The torrent was just queued on TorBox and has no playable link yet.
///
/// Atlas is a resolver, not an origin: it never serves media bytes itself and
/// never hotlinks someone else's asset. Operators who want a "please wait"
/// clip point `ATLAS_PLACEHOLDER_VIDEO_URL` at storage they control; otherwise
/// the client is told to retry.
fn preparing_response() -> axum::response::Response {
    if let Ok(url) = std::env::var("ATLAS_PLACEHOLDER_VIDEO_URL") {
        let url = url.trim();
        if url.starts_with("http") {
            return (StatusCode::FOUND, [("Location", url.to_string())]).into_response();
        }
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        [("Retry-After", "30")],
        axum::Json(serde_json::json!({
            "status": "preparing",
            "message": "Source is being cached by TorBox. Try again shortly."
        })),
    )
        .into_response()
}

/// Resolves a hash to the final TorBox CDN URL server-side, so the client is
/// redirected straight to the CDN and the API key never leaves the backend.
async fn resolve_torbox_playback(
    client: &reqwest::Client,
    hash: &str,
    api_key: &str,
    scope: &str,
    cache_id: &str,
    season: Option<u32>,
    episode: Option<u32>,
) -> Option<String> {
    // A cached torrent handle skips createtorrent, but may be stale (torrent
    // removed from the account) — on failure fall through to a fresh create.
    if let Some((torrent_id, file_id)) = cached_torrent_handle(scope, cache_id) {
        if let Some(url) = request_torbox_download(client, api_key, torrent_id, file_id).await {
            store_playback_url(scope, cache_id, &url);
            return Some(url);
        }
    }

    let (torrent_id, file_id) =
        create_torbox_torrent(client, hash, api_key, season, episode).await?;
    let url = request_torbox_download(client, api_key, torrent_id, file_id).await?;

    store_torrent_handle(scope, cache_id, torrent_id, file_id);
    store_playback_url(scope, cache_id, &url);
    Some(url)
}

async fn add_torbox_magnet(
    client: &reqwest::Client,
    hash: &str,
    api_key: &str,
) -> Option<TorboxTorrentData> {
    let magnet = format!("magnet:?xt=urn:btih:{}", hash);

    let res = client
        .post("https://api.torbox.app/v1/api/torrents/createtorrent")
        .bearer_auth(api_key)
        .form(&[("magnet", magnet.as_str())])
        .send()
        .await
        .ok()?;

    let json = res.json::<TorboxCreateResponse>().await.ok()?;
    if !json.success {
        return None;
    }
    json.data
}

async fn create_torbox_torrent(
    client: &reqwest::Client,
    hash: &str,
    api_key: &str,
    season: Option<u32>,
    episode: Option<u32>,
) -> Option<(u64, u64)> {
    let data = add_torbox_magnet(client, hash, api_key).await?;
    let torrent_id = data.torrent_id;

    let file_id = match select_best_video_file(&data.files, season, episode) {
        Some(file_id) => file_id,
        None => find_best_torbox_file_id(client, api_key, torrent_id, season, episode).await?,
    };

    Some((torrent_id, file_id))
}

async fn request_torbox_download(
    client: &reqwest::Client,
    api_key: &str,
    torrent_id: u64,
    file_id: u64,
) -> Option<String> {
    let url = format!(
        "https://api.torbox.app/v1/api/torrents/requestdl?token={}&torrent_id={}&file_id={}",
        api_key, torrent_id, file_id
    );

    let res = client.get(&url).send().await.ok()?;
    let json = res.json::<Value>().await.ok()?;
    extract_torbox_download_url(&json)
}

fn extract_torbox_download_url(json: &Value) -> Option<String> {
    if json.get("success").and_then(Value::as_bool) != Some(true) {
        return None;
    }

    let data = json.get("data")?;
    let url = match data {
        Value::String(url) => Some(url.as_str()),
        Value::Object(map) => ["url", "download_url", "link"]
            .iter()
            .find_map(|key| map.get(*key).and_then(Value::as_str)),
        _ => None,
    }?;

    if url.starts_with("http") {
        Some(url.to_string())
    } else {
        None
    }
}

fn playback_cache_id(hash: &str, season: Option<u32>, episode: Option<u32>) -> String {
    match (season, episode) {
        (Some(season), Some(episode)) => {
            format!("{}:{}:{}", hash.to_lowercase(), season, episode)
        }
        _ => hash.to_lowercase(),
    }
}

fn cached_playback_url(scope: &str, cache_id: &str) -> Option<String> {
    let key = scoped_key(scope, "playback_url_torbox", cache_id);
    get_json(&key)?.as_str().map(str::to_string)
}

fn store_playback_url(scope: &str, cache_id: &str, url: &str) {
    let key = scoped_key(scope, "playback_url_torbox", cache_id);
    set_json(key, Value::String(url.to_string()), PLAYBACK_URL_TTL);
}

fn cached_torrent_handle(scope: &str, cache_id: &str) -> Option<(u64, u64)> {
    let key = scoped_key(scope, "torbox_torrent", cache_id);
    let value = get_json(&key)?;
    let torrent_id = value.get("torrent_id")?.as_u64()?;
    let file_id = value.get("file_id")?.as_u64()?;
    Some((torrent_id, file_id))
}

fn store_torrent_handle(scope: &str, cache_id: &str, torrent_id: u64, file_id: u64) {
    let key = scoped_key(scope, "torbox_torrent", cache_id);
    set_json(
        key,
        serde_json::json!({ "torrent_id": torrent_id, "file_id": file_id }),
        TORRENT_HANDLE_TTL,
    );
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

/// What a resolve attempt produced, for playback-history purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolveOutcome {
    /// A playable link was handed to the client.
    Played,
    /// The torrent was queued on TorBox for caching. Nothing played.
    Queued,
    /// No playable link could be produced.
    Failed,
}

impl ResolveOutcome {
    /// Playback history only tracks whether a source actually played, because
    /// ranking scores sources by their success and failure counts. Queuing is
    /// neither: counting it a success would boost sources that merely made the
    /// user wait, and counting it a failure would bury sources that are fine
    /// once cached.
    fn history_record(self) -> Option<bool> {
        match self {
            ResolveOutcome::Played => Some(true),
            ResolveOutcome::Failed => Some(false),
            ResolveOutcome::Queued => None,
        }
    }
}

fn record_provider_outcome(
    history_scope: Option<&str>,
    provider: &str,
    hash: &str,
    outcome: ResolveOutcome,
) {
    let Some(success) = outcome.history_record() else {
        return;
    };

    match history_scope {
        Some(scope) => record_playback_scope(scope, provider, hash, success),
        None => record_playback(provider, hash, success),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cached_playback_url, cached_torrent_handle, extract_torbox_download_url, playback_cache_id,
        record_provider_outcome, store_playback_url, store_torrent_handle, ResolveOutcome,
    };
    use crate::engines::history::stats_for_scope;
    use serde_json::json;

    #[test]
    fn only_real_playbacks_reach_history() {
        assert_eq!(ResolveOutcome::Played.history_record(), Some(true));
        assert_eq!(ResolveOutcome::Failed.history_record(), Some(false));
        assert_eq!(ResolveOutcome::Queued.history_record(), None);
    }

    #[test]
    fn queuing_a_source_does_not_count_as_a_playback() {
        // Ranking adds playback_successes * 180 to a source's score, so a
        // queued-but-never-played source must not gain a success.
        let scope = "test-queue-outcome";
        let hash = "b1946ac92492d2347c6235b4d2611184";
        let before = stats_for_scope(scope, "TorBox", Some(hash));

        record_provider_outcome(Some(scope), "TorBox", hash, ResolveOutcome::Queued);
        let after_queue = stats_for_scope(scope, "TorBox", Some(hash));

        assert_eq!(after_queue.successes, before.successes);
        assert_eq!(after_queue.failures, before.failures);

        // A real playback on the same source still counts, so the queue path
        // is skipping the record rather than history being broken.
        record_provider_outcome(Some(scope), "TorBox", hash, ResolveOutcome::Played);
        let after_play = stats_for_scope(scope, "TorBox", Some(hash));

        assert_eq!(after_play.successes, before.successes + 1);
        assert_eq!(after_play.failures, before.failures);
    }

    #[test]
    fn extracts_cdn_url_from_requestdl_string_payload() {
        let payload = json!({
            "success": true,
            "data": "https://store-031.weur.tb-cdn.st/movie.mkv?token=signed"
        });

        assert_eq!(
            extract_torbox_download_url(&payload),
            Some("https://store-031.weur.tb-cdn.st/movie.mkv?token=signed".to_string())
        );
    }

    #[test]
    fn extracts_cdn_url_from_requestdl_object_payload() {
        let payload = json!({
            "success": true,
            "data": { "url": "https://cdn.torbox.app/movie.mkv" }
        });

        assert_eq!(
            extract_torbox_download_url(&payload),
            Some("https://cdn.torbox.app/movie.mkv".to_string())
        );
    }

    #[test]
    fn rejects_failed_or_malformed_requestdl_payloads() {
        assert_eq!(
            extract_torbox_download_url(&json!({ "success": false, "data": "https://x" })),
            None
        );
        assert_eq!(
            extract_torbox_download_url(&json!({ "success": true, "data": "not-a-url" })),
            None
        );
        assert_eq!(
            extract_torbox_download_url(&json!({ "success": true })),
            None
        );
    }

    #[test]
    fn playback_cache_id_carries_episode_context() {
        assert_eq!(playback_cache_id("ABC123", None, None), "abc123");
        assert_eq!(playback_cache_id("ABC123", Some(1), Some(2)), "abc123:1:2");
        assert_ne!(
            playback_cache_id("abc123", Some(1), Some(2)),
            playback_cache_id("abc123", Some(1), Some(3))
        );
    }

    #[test]
    fn playback_url_cache_round_trips_and_is_scope_isolated() {
        store_playback_url("tenant-a", "abc123", "https://cdn.torbox.app/a.mkv");

        assert_eq!(
            cached_playback_url("tenant-a", "abc123"),
            Some("https://cdn.torbox.app/a.mkv".to_string())
        );
        assert_eq!(cached_playback_url("tenant-b", "abc123"), None);
    }

    #[test]
    fn torrent_handle_cache_round_trips() {
        store_torrent_handle("local", "def456:1:2", 42, 7);

        assert_eq!(cached_torrent_handle("local", "def456:1:2"), Some((42, 7)));
        assert_eq!(cached_torrent_handle("other", "def456:1:2"), None);
    }

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
