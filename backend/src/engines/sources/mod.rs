pub mod torbox;
pub mod real_debrid;

use crate::engines::identity::AtlasID;
use crate::engines::metadata::MediaMetadata;

#[derive(Debug, Clone)]
pub struct SourceResult {
    pub provider_name: String,
    pub title: String,
    pub hash: Option<String>,
    pub size_bytes: Option<u64>,
    pub resolution: String, // e.g. "4K", "1080p"
    pub codec: String,      // e.g. "HEVC", "H264", "AV1"
    pub has_hdr: bool,
    pub is_cached: bool,
    pub url: Option<String>, // if instantly resolvable
}

#[async_trait::async_trait]
pub trait SourceProvider: Send + Sync {
    fn name(&self) -> &'static str;
    
    /// Search the provider for a given media
    async fn search(&self, atlas_id: &AtlasID, metadata: &MediaMetadata) -> Vec<SourceResult>;
    
    /// Resolve a specific SourceResult into a playable stream URL
    async fn resolve(&self, result: &SourceResult) -> Option<String>;
    
    /// Get the health latency of the provider in milliseconds
    async fn health(&self) -> u64;
    
    /// Returns 1-100 priority score
    fn priority(&self) -> u8;
}
