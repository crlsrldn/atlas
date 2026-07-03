use crate::api::config::{current_preferences, UserPreferences};
use crate::engines::history::{remember_candidates, stats_for, PlaybackCandidate};
use crate::engines::identity::AtlasID;
use crate::engines::metadata::get_metadata;
use crate::engines::ranking::rank_sources;
use crate::engines::sources::{
    torbox::TorBoxProvider, ProviderHealthStatus, SourceProvider,
};
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

pub async fn resolve_detailed_streams(atlas_id: AtlasID) -> Vec<DetailedStream> {
    let prefs = current_preferences();
    resolve_detailed_streams_with_preferences(atlas_id, prefs, "local", None).await
}

pub async fn resolve_detailed_streams_with_preferences(
    atlas_id: AtlasID,
    prefs: UserPreferences,
    history_scope: &str,
    install_token: Option<&str>,
) -> Vec<DetailedStream> {
    // 1. Fetch Metadata
    let metadata = get_metadata(&atlas_id).await;

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
        let health = provider.health().await;
        crate::engines::telemetry::log_event(
            "provider_health",
            serde_json::json!({
                "provider": health.provider_name,
                "configured": health.configured,
                "healthy": health.is_healthy(),
                "latency_ms": health.latency_ms,
                "priority": health.priority,
                "message": health.message
            }),
        );

        if !matches!(health.status, ProviderHealthStatus::Ok) {
            continue;
        }
        search_futures.push(provider.search(&atlas_id, &metadata));
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
            if let Some(existing) = unique_results.get_mut(hash) {
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
                unique_results.insert(hash.clone(), res);
            }
        }
    }

    // 4. Rank the results based on user preferences and PRD rules
    let ranked = rank_sources(unique_results.into_values().collect(), &prefs);
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
        "{}/stremio/{}/resolve/{}/{}/play.mp4",
        base_url.trim_end_matches('/'),
        token,
        provider_slug,
        hash
    );

    if let Some((season, episode)) = atlas_id.season_episode() {
        final_url.push_str(&format!("?season={}&episode={}", season, episode));
    }

    Some(final_url)
}

pub async fn resolve_stream_for_tenant(
    atlas_id: AtlasID,
    prefs: UserPreferences,
    history_scope: &str,
    install_token: &str,
) -> Vec<StremioStream> {
    resolve_detailed_streams_with_preferences(atlas_id, prefs, history_scope, Some(install_token))
        .await
        .into_iter()
        .take(5)
        .map(stremio_stream_from_detail)
        .collect()
}

pub async fn resolve_stream(atlas_id: AtlasID) -> Vec<StremioStream> {
    resolve_detailed_streams(atlas_id)
        .await
        .into_iter()
        .take(5)
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

    let description = format!("{}\n{}\n{}", stream.title, specs.join(" | "), explanation);

    StremioStream {
        name: Some(format!("Atlas\n{}", stream.provider_name)),
        description: Some(description),
        url: stream.url,
    }
}
