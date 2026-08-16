pub mod torbox;

use crate::engines::identity::AtlasID;
use crate::engines::metadata::MediaMetadata;

#[derive(Debug, Clone)]
pub enum ProviderHealthStatus {
    Ok,
    NotConfigured,
    Error,
}

#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider_name: String,
    pub configured: bool,
    pub status: ProviderHealthStatus,
    pub latency_ms: Option<u64>,
    pub priority: u8,
    pub message: String,
}

impl ProviderHealth {
    pub fn ok(provider_name: &str, latency_ms: u64, priority: u8) -> Self {
        Self {
            provider_name: provider_name.to_string(),
            configured: true,
            status: ProviderHealthStatus::Ok,
            latency_ms: Some(latency_ms),
            priority,
            message: "Connection verified.".to_string(),
        }
    }

    pub fn not_configured(provider_name: &str, priority: u8) -> Self {
        Self {
            provider_name: provider_name.to_string(),
            configured: false,
            status: ProviderHealthStatus::NotConfigured,
            latency_ms: None,
            priority,
            message: "No API key configured.".to_string(),
        }
    }

    pub fn error(
        provider_name: &str,
        latency_ms: Option<u64>,
        priority: u8,
        message: impl Into<String>,
    ) -> Self {
        Self {
            provider_name: provider_name.to_string(),
            configured: true,
            status: ProviderHealthStatus::Error,
            latency_ms,
            priority,
            message: message.into(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, ProviderHealthStatus::Ok)
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceResult {
    pub provider_name: String,
    pub provider_priority: u8,
    pub provider_latency_ms: Option<u64>,
    pub title: String,
    pub raw_title: String,
    pub hash: Option<String>,
    pub size_bytes: Option<u64>,
    pub bitrate_mbps: Option<f32>,
    pub resolution: String, // e.g. "4K", "1080p"
    pub codec: String,      // e.g. "HEVC", "H264", "AV1"
    /// Container extension ("mkv", "mp4"), when the release name reveals one.
    ///
    /// Defaulted because these results are cached in Redis for 20 minutes: a
    /// deploy that added this field without a default would fail to deserialize
    /// every warm entry and send each request back to a live provider search.
    #[serde(default)]
    pub container: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<String>,
    pub has_hdr: bool,
    pub has_dolby_vision: bool,
    pub has_subtitles: bool,
    pub is_cached: bool,
    pub url: Option<String>, // if instantly resolvable
    pub release_group: Option<String>,
    pub verification_score: u8,
    pub verification_reasons: Vec<String>,
    pub playback_successes: u32,
    pub playback_failures: u32,
}

#[async_trait::async_trait]
pub trait SourceProvider: Send + Sync {
    fn name(&self) -> &'static str;

    /// Search the provider for a given media
    async fn search(&self, atlas_id: &AtlasID, metadata: &MediaMetadata) -> Vec<SourceResult>;

    /// Resolve a specific SourceResult into a playable stream URL
    async fn resolve(&self, result: &SourceResult) -> Option<String>;

    /// Get structured health for the provider.
    async fn health(&self) -> ProviderHealth;

    /// Returns 1-100 priority score
    fn priority(&self) -> u8;
}

#[cfg(test)]
mod tests {
    use super::SourceResult;

    /// Source results live in Redis for 20 minutes, so a deploy always reads
    /// back entries written by the previous build. A new field that cannot
    /// deserialize from an older entry would silently send every warm request
    /// back to a live provider search until the cache turned over.
    #[test]
    fn deserializes_results_cached_before_container_existed() {
        let cached = serde_json::json!({
            "provider_name": "TorBox",
            "provider_priority": 90,
            "provider_latency_ms": 250,
            "title": "The Matrix (4K)",
            "raw_title": "The.Matrix.1999.2160p.mkv",
            "hash": "abc123",
            "size_bytes": 24_300_000_000u64,
            "bitrate_mbps": 32.4,
            "resolution": "4K",
            "codec": "HEVC",
            "audio_codec": "TrueHD",
            "audio_channels": "7.1",
            "has_hdr": true,
            "has_dolby_vision": true,
            "has_subtitles": false,
            "is_cached": true,
            "url": "https://example.invalid/play",
            "release_group": "TERMINAL",
            "verification_score": 85,
            "verification_reasons": ["✨ Verified Hash"],
            "playback_successes": 3,
            "playback_failures": 0
        });

        let parsed: SourceResult =
            serde_json::from_value(cached).expect("entry without `container` must still parse");

        assert_eq!(parsed.container, None);
        assert_eq!(parsed.resolution, "4K");
    }
}
