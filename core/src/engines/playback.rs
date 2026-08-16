use crate::api::config::{current_preferences, UserPreferences};
use crate::engines::cache::{
    get_json, scoped_key, set_json, FAILED_PROVIDER_TTL, PROVIDER_HEALTH_TTL, SOURCE_RESULTS_TTL,
};
use crate::engines::history::{remember_candidates, stats_for, PlaybackCandidate};
use crate::engines::identity::AtlasID;
use crate::engines::metadata::get_metadata;
use crate::engines::ranking::rank_sources;
use crate::engines::sources::{torbox::TorBoxProvider, SourceProvider, SourceResult};

use crate::engines::verification::verify_source;
use futures::future::join_all;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StremioStream {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedStream {
    pub title: String,
    /// The provider's release name. Kept because it is the only place a
    /// container or a precise release string survives; `title` is synthesized.
    #[serde(default)]
    pub raw_title: String,
    /// Container extension ("mkv", "mp4"), when known.
    #[serde(default)]
    pub container: Option<String>,
    pub provider_name: String,
    pub url: String,
    pub hash: Option<String>,
    pub score: u64,
    pub confidence: u8,
    pub reasons: Vec<String>,
    pub resolution: String,
    pub video_codec: String,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<String>,
    pub bitrate_mbps: Option<f32>,
    pub has_hdr: bool,
    pub has_dolby_vision: bool,
    pub has_subtitles: bool,
    pub provider_latency_ms: Option<u64>,
    pub playback_successes: u32,
    pub playback_failures: u32,
    pub is_cached: bool,
    pub release_group: Option<String>,
    pub size_bytes: Option<u64>,
}

pub fn media_key(atlas_id: &AtlasID) -> String {
    match atlas_id {
        AtlasID::IMDb {
            id,
            season: Some(season),
            episode: Some(episode),
        } => format!("{}:{}:{}", id, season, episode),
        AtlasID::IMDb { id, .. } => id.clone(),
        AtlasID::TMDB(id) => format!("tmdb:{}", id),
    }
}

/// Reads cached provider search results.
///
/// Redis is shared across machines and preferred when configured. It is
/// optional though — `get_redis()` yields None when UPSTASH_REDIS_URL is unset,
/// and previously that made this cache silently do nothing — so the in-process
/// cache backs it up and keeps a single machine from re-searching on every
/// request.
async fn cached_source_results(cache_key: &str) -> Option<Vec<SourceResult>> {
    if let Some(mut redis_client) = crate::engines::redis::get_redis() {
        let cached: Result<String, _> = redis::cmd("GET")
            .arg(cache_key)
            .query_async(&mut redis_client)
            .await;

        if let Ok(cached_json) = cached {
            if let Ok(results) = serde_json::from_str::<Vec<SourceResult>>(&cached_json) {
                return Some(results);
            }
        }
    }

    let cached = get_json(cache_key)?;
    serde_json::from_value::<Vec<SourceResult>>(cached).ok()
}

async fn store_source_results(cache_key: &str, results: &[SourceResult]) {
    if let Ok(value) = serde_json::to_value(results) {
        set_json(cache_key.to_string(), value, SOURCE_RESULTS_TTL);
    }

    let Some(mut redis_client) = crate::engines::redis::get_redis() else {
        return;
    };
    let Ok(json) = serde_json::to_string(results) else {
        return;
    };

    let stored: Result<(), redis::RedisError> = redis::cmd("SETEX")
        .arg(cache_key)
        .arg(SOURCE_RESULTS_TTL.as_secs())
        .arg(json)
        .query_async(&mut redis_client)
        .await;

    if let Err(e) = stored {
        tracing::error!("Redis SETEX failed for {}: {:?}", cache_key, e);
    }
}

/// Checks a provider's health, reusing a recent verdict rather than probing the
/// provider's API before every single search.
///
/// A healthy verdict is cached for PROVIDER_HEALTH_TTL. An unhealthy one is
/// cached for the shorter FAILED_PROVIDER_TTL, so a provider that recovers is
/// picked back up quickly while a broken one stops being retried on every
/// request. The key includes a fingerprint of the credential so that fixing a
/// bad API key takes effect immediately instead of waiting out the TTL.
async fn provider_is_healthy(provider: &dyn SourceProvider, api_key: &str) -> bool {
    let cache_key = scoped_key(
        "provider_health",
        provider.name(),
        &credential_fingerprint(api_key),
    );

    if let Some(cached) = get_json(&cache_key) {
        if let Some(healthy) = cached.as_bool() {
            return healthy;
        }
    }

    let health = provider.health().await;
    let healthy = health.is_healthy();

    crate::engines::telemetry::log_event(
        "provider_health",
        serde_json::json!({
            "provider": health.provider_name,
            "configured": health.configured,
            "healthy": healthy,
            "latency_ms": health.latency_ms,
            "priority": health.priority,
            "message": health.message
        }),
    );

    let ttl = if healthy {
        PROVIDER_HEALTH_TTL
    } else {
        FAILED_PROVIDER_TTL
    };
    set_json(cache_key, serde_json::Value::Bool(healthy), ttl);

    healthy
}

/// A short, non-reversible tag for a credential, so cache keys can change when
/// the key changes without ever holding the key itself.
fn credential_fingerprint(api_key: &str) -> String {
    let mut hash: u64 = 14_695_981_039_346_656_037;
    for byte in api_key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    format!("{:016x}", hash)
}

pub async fn resolve_detailed_streams(atlas_id: AtlasID) -> Vec<DetailedStream> {
    let prefs = current_preferences();
    resolve_detailed_streams_with_preferences(atlas_id, prefs, false, "local", None).await
}

pub async fn resolve_detailed_streams_with_preferences(
    atlas_id: AtlasID,
    prefs: UserPreferences,
    monetization_enabled: bool,
    history_scope: &str,
    install_token: Option<&str>,
) -> Vec<DetailedStream> {
    // 1. Fetch Metadata
    let metadata = get_metadata(&atlas_id).await;

    // Apply AI Device Profile constraints

    // 2. Initialize Source Plugins
    let torbox = TorBoxProvider {
        api_key: prefs.torbox_api_key.clone(),
    };

    let mut providers: Vec<&dyn SourceProvider> = Vec::new();
    if !prefs.torbox_api_key.is_empty() {
        providers.push(&torbox);
    }

    let mut search_futures = Vec::new();
    for provider in providers {
        if !provider_is_healthy(provider, &prefs.torbox_api_key).await {
            continue;
        }

        let atlas_id_str = media_key(&atlas_id);
        let provider_name = provider.name().to_string();
        let atlas_id_clone = atlas_id.clone();
        let metadata_clone = metadata.clone();

        search_futures.push(async move {
            let cache_key = format!(
                "atlas:sources:{}:{}",
                provider_name.to_lowercase(),
                atlas_id_str
            );

            if let Some(results) = cached_source_results(&cache_key).await {
                return results;
            }

            let results = provider.search(&atlas_id_clone, &metadata_clone).await;

            if !results.is_empty() {
                store_source_results(&cache_key, &results).await;
            }

            results
        });
    }

    let results = join_all(search_futures).await;
    let mut all_results = Vec::new();
    for mut r in results {
        all_results.append(&mut r);
    }

    for source in &mut all_results {
        let verification = verify_source(source, &metadata);
        source.verification_score = verification.confidence;
        source.verification_reasons = verification.reasons;
        let stats = if history_scope == "local" {
            stats_for(&source.provider_name, source.hash.as_deref())
        } else {
            crate::engines::history::stats_for_scope(
                history_scope,
                &source.provider_name,
                source.hash.as_deref(),
            )
        };
        source.playback_successes = stats.successes;
        source.playback_failures = stats.failures;
    }

    // Deduplicate by hash, merging providers
    let mut unique_results: std::collections::HashMap<
        String,
        crate::engines::sources::SourceResult,
    > = std::collections::HashMap::new();
    for res in all_results {
        if let Some(hash) = &res.hash {
            let hash_key = hash.to_lowercase();
            if let Some(existing) = unique_results.get_mut(&hash_key) {
                if !existing.provider_name.contains(&res.provider_name) {
                    existing.provider_name =
                        format!("{} + {}", existing.provider_name, res.provider_name);
                }
                if res.provider_priority > existing.provider_priority {
                    existing.url = res.url;
                    existing.provider_priority = res.provider_priority;
                    existing.provider_latency_ms = res.provider_latency_ms;
                }
            } else {
                unique_results.insert(hash_key, res);
            }
        }
    }

    // 4. Rank the results based on user preferences and PRD rules
    let mut ranked = rank_sources(
        unique_results.into_values().collect(),
        &prefs,
        monetization_enabled,
    );

    // 5. Visually deduplicate identical looking streams to avoid UI clutter
    let mut seen_visuals = std::collections::HashSet::new();
    ranked.retain(|entry| {
        let visual_key = format!(
            "{}-{}-{}-{}-{}",
            entry.source.resolution,
            entry.source.codec,
            entry.source.audio_codec.as_deref().unwrap_or("none"),
            entry.source.release_group.as_deref().unwrap_or("none"),
            entry.source.is_cached
        );
        seen_visuals.insert(visual_key)
    });
    let candidates: Vec<PlaybackCandidate> = ranked
        .iter()
        .filter_map(|entry| {
            Some(PlaybackCandidate {
                provider: entry.source.provider_name.clone(),
                hash: entry.source.hash.clone()?,
                url: hosted_or_local_url(&atlas_id, &entry.source, install_token)?,
                score: entry.score,
            })
        })
        .collect();

    if history_scope == "local" {
        remember_candidates(&media_key(&atlas_id), candidates);
    } else {
        crate::engines::history::remember_candidates_scope(
            history_scope,
            &media_key(&atlas_id),
            candidates,
        );
    }

    ranked
        .into_iter()
        .filter(|r| r.score > 0)
        .filter_map(|entry| {
            let url = hosted_or_local_url(&atlas_id, &entry.source, install_token)?;
            Some(DetailedStream {
                title: entry.source.title.clone(),
                raw_title: entry.source.raw_title.clone(),
                container: entry.source.container.clone(),
                provider_name: entry.source.provider_name.clone(),
                url,
                hash: entry.source.hash.clone(),
                score: entry.score,
                confidence: entry.source.verification_score,
                reasons: entry.source.verification_reasons.clone(),
                resolution: entry.source.resolution.clone(),
                video_codec: entry.source.codec.clone(),
                audio_codec: entry.source.audio_codec.clone(),
                audio_channels: entry.source.audio_channels.clone(),
                bitrate_mbps: entry.source.bitrate_mbps,
                has_hdr: entry.source.has_hdr,
                has_dolby_vision: entry.source.has_dolby_vision,
                has_subtitles: entry.source.has_subtitles,
                provider_latency_ms: entry.source.provider_latency_ms,
                playback_successes: entry.source.playback_successes,
                playback_failures: entry.source.playback_failures,
                is_cached: entry.source.is_cached,
                release_group: entry.source.release_group.clone(),
                size_bytes: entry.source.size_bytes,
            })
        })
        .collect()
}

fn hosted_or_local_url(
    atlas_id: &AtlasID,
    source: &crate::engines::sources::SourceResult,
    install_token: Option<&str>,
) -> Option<String> {
    let url = source.url.clone()?;
    let Some(token) = install_token else {
        return Some(url);
    };
    let hash = source.hash.as_ref()?;
    let provider_slug = match source.provider_name.as_str() {
        provider if provider.contains("TorBox") => "torbox",
        _ => return Some(url),
    };
    let base_url = std::env::var("ATLAS_PUBLIC_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    let mut final_url = format!(
        "{}/stremio/{}/resolve/{}/{}/play.mp4?cached={}",
        base_url.trim_end_matches('/'),
        token,
        provider_slug,
        hash,
        source.is_cached
    );

    if let Some((season, episode)) = atlas_id.season_episode() {
        final_url.push_str(&format!("&season={}&episode={}", season, episode));
    }

    Some(final_url)
}

pub async fn resolve_stream_for_tenant(
    atlas_id: AtlasID,
    prefs: UserPreferences,
    monetization_enabled: bool,
    history_scope: &str,
    install_token: &str,
) -> Vec<StremioStream> {
    let limit = if prefs.stream_limit > 0 {
        prefs.stream_limit as usize
    } else {
        5
    };
    resolve_detailed_streams_with_preferences(
        atlas_id,
        prefs,
        monetization_enabled,
        history_scope,
        Some(install_token),
    )
    .await
    .into_iter()
    .take(limit)
    .map(stremio_stream_from_detail)
    .collect()
}

pub async fn resolve_stream(atlas_id: AtlasID) -> Vec<StremioStream> {
    let prefs = current_preferences();
    let limit = if prefs.stream_limit > 0 {
        prefs.stream_limit as usize
    } else {
        5
    };
    resolve_detailed_streams(atlas_id)
        .await
        .into_iter()
        .take(limit)
        .map(stremio_stream_from_detail)
        .collect()
}

fn stremio_stream_from_detail(stream: DetailedStream) -> StremioStream {
    let explanation = stream
        .reasons
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    let mut specs = Vec::new();
    specs.push(stream.resolution.clone());
    specs.push(stream.video_codec.clone());
    if let Some(audio) = stream.audio_codec {
        specs.push(audio);
    }
    if let Some(channels) = stream.audio_channels {
        specs.push(channels);
    }
    if stream.has_hdr {
        specs.push("HDR".to_string());
    }
    if stream.has_dolby_vision {
        specs.push("DV".to_string());
    }
    if let Some(mbps) = stream.bitrate_mbps {
        specs.push(format!("{:.1} Mbps", mbps));
    }
    if let Some(bytes) = stream.size_bytes {
        let gb = bytes as f64 / 1_073_741_824.0;
        specs.push(format!("{:.2} GB", gb));
    }
    if let Some(rg) = stream.release_group {
        specs.push(rg);
    }

    let description = format!("{}\n{}\n{}", stream.title, specs.join(" | "), explanation);

    let prefix = if stream.is_cached {
        "⚡️ "
    } else {
        "⬇️ [Uncached] "
    };
    let name = format!("Atlas\n{}{}", prefix, stream.provider_name);

    StremioStream {
        name: Some(name),
        description: Some(description),
        url: stream.url,
    }
}

#[cfg(test)]
mod tests {
    use super::credential_fingerprint;

    #[test]
    fn credential_fingerprint_changes_with_the_key_and_hides_it() {
        let fingerprint = credential_fingerprint("tb_live_secret");

        assert_ne!(fingerprint, credential_fingerprint("tb_live_rotated"));
        assert_eq!(fingerprint, credential_fingerprint("tb_live_secret"));
        assert!(!fingerprint.contains("secret"));
    }
}
