use crate::engines::identity::AtlasID;
use crate::engines::metadata::MediaMetadata;
use crate::engines::sources::{ProviderHealth, SourceProvider, SourceResult};
use async_trait::async_trait;
use reqwest;

pub struct TorBoxProvider {
    pub api_key: String,
}

#[async_trait]
impl SourceProvider for TorBoxProvider {
    fn name(&self) -> &'static str {
        "TorBox"
    }

    async fn search(&self, _atlas_id: &AtlasID, metadata: &MediaMetadata) -> Vec<SourceResult> {
        if self.api_key.is_empty() || metadata.torrents.is_empty() {
            return vec![];
        }

        // 1. Gather all hashes
        let hashes: Vec<String> = metadata.torrents.iter().map(|t| t.hash.clone()).collect();
        let hash_param = hashes.join(",");

        let url = format!("https://api.torbox.app/v1/api/torrents/checkcached?hash={}&format=list&list_files=false", hash_param);

        let client = reqwest::Client::new();
        let mut cached_hashes = Vec::new();

        let mut error_msg: Option<String> = None;

        let start_time = std::time::Instant::now();
        match client.get(&url).bearer_auth(&self.api_key).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    if let Ok(text) = res.text().await {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            if json["success"].as_bool() == Some(true) {
                                if let Some(arr) = json["data"].as_array() {
                                    for item in arr {
                                        if let Some(h) = item.as_str() {
                                            cached_hashes.push(h.to_lowercase());
                                        } else if let Some(obj) = item.as_object() {
                                            if let Some(h) =
                                                obj.get("hash").and_then(|h| h.as_str())
                                            {
                                                cached_hashes.push(h.to_lowercase());
                                            }
                                        }
                                    }
                                } else if let Some(obj) = json["data"].as_object() {
                                    for key in obj.keys() {
                                        cached_hashes.push(key.to_lowercase());
                                    }
                                }
                            } else {
                                let detail = json["detail"]
                                    .as_str()
                                    .unwrap_or("TorBox API returned success=false");
                                tracing::warn!("Torbox checkcached failed: {}", detail);
                                error_msg = Some(detail.to_string());
                            }
                        } else {
                            error_msg = Some("Failed to parse TorBox JSON".to_string());
                        }
                    }
                } else {
                    error_msg = Some(format!("HTTP Error {}", res.status()));
                }
            }
            Err(e) => {
                error_msg = Some(e.to_string());
            }
        }
        let latency_ms = start_time.elapsed().as_millis() as u64;

        crate::engines::telemetry::log_event(
            "torbox_cache_check",
            serde_json::json!({
                "latency_ms": latency_ms,
                "hashes_checked": hashes.len(),
                "hashes_cached": cached_hashes.len(),
                "error": error_msg
            }),
        );

        // 2. Filter metadata torrents by cached hashes
        let mut results = Vec::new();
        for t in &metadata.torrents {
            let is_cached = cached_hashes.contains(&t.hash.to_lowercase());

            results.push(SourceResult {
                provider_name: self.name().to_string(),
                provider_priority: self.priority(),
                provider_latency_ms: Some(latency_ms),
                title: format!("{} ({})", metadata.title, t.quality),
                raw_title: t.raw_title.clone(),
                hash: Some(t.hash.clone()),
                size_bytes: Some(t.size_bytes),
                bitrate_mbps: t.bitrate_mbps,
                resolution: t.quality.clone(),
                codec: t.video_codec.clone(),
                audio_codec: t.audio_codec.clone(),
                audio_channels: t.audio_channels.clone(),
                has_hdr: t.has_hdr,
                has_dolby_vision: t.has_dolby_vision,
                has_subtitles: t.has_subtitles,
                is_cached,
                // Use the local callback so we can lazily resolve the torrent on click
                url: Some(format!("http://127.0.0.1:3000/resolve/torbox/{}", t.hash)),
                release_group: t.release_group.clone(),
                verification_score: 0,
                verification_reasons: vec![],
                playback_successes: 0,
                playback_failures: 0,
            });
        }

        results
    }

    async fn resolve(&self, result: &SourceResult) -> Option<String> {
        // Since we are returning the local proxy URL in `search`, this resolve
        // doesn't need to do the heavy lifting. The heavy lifting is done in `/resolve/torbox/:hash`.
        result.url.clone()
    }

    async fn health(&self) -> ProviderHealth {
        if self.api_key.is_empty() {
            return ProviderHealth::not_configured(self.name(), self.priority());
        }

        let started = std::time::Instant::now();
        match reqwest::Client::new()
            .get("https://api.torbox.app/v1/api/user/me")
            .bearer_auth(&self.api_key)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => ProviderHealth::ok(
                self.name(),
                started.elapsed().as_millis() as u64,
                self.priority(),
            ),
            Ok(response) => ProviderHealth::error(
                self.name(),
                Some(started.elapsed().as_millis() as u64),
                self.priority(),
                format!("HTTP {}", response.status()),
            ),
            Err(err) => ProviderHealth::error(
                self.name(),
                Some(started.elapsed().as_millis() as u64),
                self.priority(),
                err.to_string(),
            ),
        }
    }

    fn priority(&self) -> u8 {
        90
    }
}
