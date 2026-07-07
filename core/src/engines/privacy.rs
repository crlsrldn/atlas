use serde_json::Value;

const SENSITIVE_KEYS: &[&str] = &[
    "api_key",
    "apikey",
    "authorization",
    "download",
    "hash",
    "key",
    "link",
    "magnet",
    "password",
    "secret",
    "token",
    "url",
];

pub fn media_kind_from_stremio_id(stremio_id: &str) -> &'static str {
    if stremio_id.contains(':') {
        "series_episode"
    } else {
        "title"
    }
}

pub fn redact_json(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                redact_json(item);
            }
        }
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if is_sensitive_key(key) {
                    *child = Value::String("[redacted]".to_string());
                } else {
                    redact_json(child);
                }
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    SENSITIVE_KEYS
        .iter()
        .any(|sensitive| normalized.contains(sensitive))
}

#[cfg(test)]
mod tests {
    use super::{media_kind_from_stremio_id, redact_json};
    use serde_json::json;

    #[test]
    fn redacts_nested_sensitive_telemetry_fields() {
        let mut payload = json!({
            "provider": "torbox",
            "hash": "abcdef",
            "nested": {
                "download_url": "https://example.test/file",
                "streams_found": 2
            }
        });

        redact_json(&mut payload);

        assert_eq!(payload["hash"], "[redacted]");
        assert_eq!(payload["nested"]["download_url"], "[redacted]");
        assert_eq!(payload["nested"]["streams_found"], 2);
    }

    #[test]
    fn classifies_series_episode_ids_without_returning_identifier() {
        assert_eq!(
            media_kind_from_stremio_id("tt0944947:1:2"),
            "series_episode"
        );
        assert_eq!(media_kind_from_stremio_id("tt0133093"), "title");
    }
}
