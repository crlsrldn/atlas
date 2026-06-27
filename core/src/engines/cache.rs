use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const METADATA_TTL: Duration = Duration::from_secs(24 * 60 * 60);
pub const PROVIDER_HEALTH_TTL: Duration = Duration::from_secs(5 * 60);
pub const SOURCE_RESULTS_TTL: Duration = Duration::from_secs(20 * 60);
pub const FAILED_PROVIDER_TTL: Duration = Duration::from_secs(2 * 60);
pub const PLAYBACK_URL_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
struct CacheEntry {
    value: Value,
    expires_at: Instant,
}

static CACHE: Lazy<Arc<Mutex<HashMap<String, CacheEntry>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub fn get_json(key: &str) -> Option<Value> {
    let mut cache = CACHE.lock().unwrap();
    let entry = cache.get(key)?;
    if Instant::now() >= entry.expires_at {
        cache.remove(key);
        return None;
    }
    Some(entry.value.clone())
}

pub fn set_json(key: impl Into<String>, value: Value, ttl: Duration) {
    CACHE.lock().unwrap().insert(
        key.into(),
        CacheEntry {
            value,
            expires_at: Instant::now() + ttl,
        },
    );
}

pub fn scoped_key(scope: &str, purpose: &str, id: &str) -> String {
    format!("{}:{}:{}", scope, purpose, id)
}

#[cfg(test)]
mod tests {
    use super::{get_json, scoped_key, set_json};
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn cache_keys_are_scoped() {
        assert_ne!(
            scoped_key("tenant-a", "sources", "tt0133093"),
            scoped_key("tenant-b", "sources", "tt0133093")
        );
    }

    #[test]
    fn expired_entries_are_not_returned() {
        set_json("expired", json!({"ok": true}), Duration::from_millis(0));

        assert!(get_json("expired").is_none());
    }
}
