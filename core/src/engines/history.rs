use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

const HISTORY_PATH: &str = "playback_history.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaybackStats {
    pub successes: u32,
    pub failures: u32,
    pub last_success_at: Option<String>,
    pub last_failure_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackCandidate {
    pub provider: String,
    pub hash: String,
    pub url: String,
    pub score: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PlaybackHistory {
    stats: HashMap<String, PlaybackStats>,
    candidates: HashMap<String, Vec<PlaybackCandidate>>,
}

static HISTORY: Lazy<Arc<Mutex<PlaybackHistory>>> =
    Lazy::new(|| Arc::new(Mutex::new(load_history().unwrap_or_default())));

pub fn stats_for(provider: &str, hash: Option<&str>) -> PlaybackStats {
    stats_for_scope("local", provider, hash)
}

pub fn stats_for_scope(scope: &str, provider: &str, hash: Option<&str>) -> PlaybackStats {
    let Some(hash) = hash else {
        return PlaybackStats::default();
    };
    let key = scoped_source_key(scope, provider, hash);
    HISTORY
        .lock()
        .unwrap()
        .stats
        .get(&key)
        .cloned()
        .unwrap_or_default()
}

pub fn record_playback(provider: &str, hash: &str, success: bool) {
    record_playback_scope("local", provider, hash, success);
}

pub fn record_playback_scope(scope: &str, provider: &str, hash: &str, success: bool) {
    let key = scoped_source_key(scope, provider, hash);
    {
        let mut history = HISTORY.lock().unwrap();
        let stats = history.stats.entry(key).or_default();
        let now = chrono::Utc::now().to_rfc3339();
        if success {
            stats.successes += 1;
            stats.last_success_at = Some(now);
        } else {
            stats.failures += 1;
            stats.last_failure_at = Some(now);
        }
    }
    save_current_history();
}

pub fn remember_candidates(media_key: &str, candidates: Vec<PlaybackCandidate>) {
    remember_candidates_scope("local", media_key, candidates);
}

pub fn remember_candidates_scope(scope: &str, media_key: &str, candidates: Vec<PlaybackCandidate>) {
    {
        let mut history = HISTORY.lock().unwrap();
        history
            .candidates
            .insert(scoped_media_key(scope, media_key), candidates);
    }
    save_current_history();
}

pub fn fallback_candidates(media_key: &str, failed_url: &str) -> Vec<PlaybackCandidate> {
    fallback_candidates_scope("local", media_key, failed_url)
}

pub fn fallback_candidates_scope(
    scope: &str,
    media_key: &str,
    failed_url: &str,
) -> Vec<PlaybackCandidate> {
    HISTORY
        .lock()
        .unwrap()
        .candidates
        .get(&scoped_media_key(scope, media_key))
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|candidate| candidate.url != failed_url)
        .collect()
}

pub fn media_key_from_hash(hash: &str) -> Option<String> {
    media_key_from_hash_scope("local", hash)
}

pub fn media_key_from_hash_scope(scope: &str, hash: &str) -> Option<String> {
    let history = HISTORY.lock().unwrap();
    history
        .candidates
        .iter()
        .find_map(|(media_key, candidates)| {
            if !media_key.starts_with(&format!("{}:", scope)) {
                return None;
            }
            candidates
                .iter()
                .any(|candidate| candidate.hash.eq_ignore_ascii_case(hash))
                .then(|| {
                    media_key
                        .trim_start_matches(&format!("{}:", scope))
                        .to_string()
                })
        })
}

pub fn source_key(provider: &str, hash: &str) -> String {
    format!(
        "{}:{}",
        provider.to_lowercase().replace(' ', "_"),
        hash.to_lowercase()
    )
}

pub fn scoped_source_key(scope: &str, provider: &str, hash: &str) -> String {
    format!("{}:{}", scope, source_key(provider, hash))
}

fn scoped_media_key(scope: &str, media_key: &str) -> String {
    format!("{}:{}", scope, media_key)
}

fn load_history() -> Option<PlaybackHistory> {
    let data = fs::read_to_string(HISTORY_PATH).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_current_history() {
    let Ok(history) = HISTORY.lock() else {
        return;
    };
    if let Ok(json) = serde_json::to_string_pretty(&*history) {
        let _ = fs::write(HISTORY_PATH, json);
    }
}

#[cfg(test)]
mod tests {
    use super::source_key;

    #[test]
    fn source_keys_are_normalized() {
        assert_eq!(source_key("Test Provider", "ABC"), "test_provider:abc");
    }

    #[test]
    fn scoped_source_keys_keep_tenants_separate() {
        assert_ne!(
            super::scoped_source_key("tenant-a", "Test Provider", "ABC"),
            super::scoped_source_key("tenant-b", "Test Provider", "ABC")
        );
    }
}
