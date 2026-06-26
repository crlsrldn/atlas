use async_trait::async_trait;
use crate::engines::sources::{SourceProvider, SourceResult};
use crate::engines::identity::AtlasID;
use crate::engines::metadata::MediaMetadata;
use reqwest;
use serde_json::Value;

pub struct RealDebridProvider {
    pub api_key: String,
}

#[async_trait]
impl SourceProvider for RealDebridProvider {
    fn name(&self) -> &'static str {
        "Real Debrid"
    }

    async fn search(&self, _atlas_id: &AtlasID, metadata: &MediaMetadata) -> Vec<SourceResult> {
        if self.api_key.is_empty() || metadata.torrents.is_empty() {
            return vec![];
        }

        let hashes: Vec<String> = metadata.torrents.iter().map(|t| t.hash.clone()).collect();
        // RD allows multiple hashes separated by slashes /
        let hash_param = hashes.join("/");

        let url = format!("https://api.real-debrid.com/rest/1.0/torrents/instantAvailability/{}", hash_param);
        
        let client = reqwest::Client::new();
        let mut cached_hashes = Vec::new();
        let mut cache_check_succeeded = false;

        if let Ok(res) = client.get(&url).bearer_auth(&self.api_key).send().await {
            if res.status().is_success() {
                cache_check_succeeded = true;
                if let Ok(json) = res.json::<Value>().await {
                    if let Some(obj) = json.as_object() {
                        for (hash, data) in obj {
                            if let Some(rd_data) = data.get("rd").and_then(|rd| rd.as_array()) {
                                if !rd_data.is_empty() {
                                    cached_hashes.push(hash.to_lowercase());
                                }
                            }
                        }
                    }
                }
            } else {
                tracing::error!("Real Debrid cache check failed: {}", res.status());
            }
        } else {
            tracing::error!("Real Debrid network request failed.");
        }
        
        tracing::info!("Real Debrid found {} cached hashes out of {}", cached_hashes.len(), hashes.len());

        if !cache_check_succeeded {
            return vec![];
        }

        let mut results = Vec::new();
        
        for t in &metadata.torrents {
            let is_cached = cached_hashes.contains(&t.hash.to_lowercase());
            
            if is_cached {
                results.push(SourceResult {
                    provider_name: self.name().to_string(),
                    title: format!("{} ({})", metadata.title, t.quality),
                    hash: Some(t.hash.clone()),
                    size_bytes: Some(t.size_bytes),
                    resolution: t.quality.clone(),
                    codec: t.video_codec.clone(),
                    has_hdr: t.has_hdr,
                    is_cached: true, // Optimistically assume true
                    url: Some(format!("http://127.0.0.1:3000/resolve/realdebrid/{}", t.hash)),
                });
            }
        }

        results
    }

    async fn resolve(&self, result: &SourceResult) -> Option<String> {
        result.url.clone()
    }

    async fn health(&self) -> u64 {
        15 // ping representation
    }

    fn priority(&self) -> u8 {
        95 // High priority for RD
    }
}
