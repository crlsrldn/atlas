use crate::api::config::current_preferences;
use crate::engines::identity::AtlasID;
use crate::engines::metadata::get_metadata;
use crate::engines::ranking::rank_sources;
use crate::engines::sources::{
    real_debrid::RealDebridProvider, torbox::TorBoxProvider, ProviderHealthStatus, SourceProvider,
};
use crate::engines::verification::verify_source;
use futures::future::join_all;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StremioStream {
    pub title: String,
    pub url: String,
}

pub async fn resolve_stream(atlas_id: AtlasID) -> Vec<StremioStream> {
    let prefs = current_preferences();

    // 1. Fetch Metadata
    let metadata = get_metadata(&atlas_id).await;

    // 2. Initialize Source Plugins
    let torbox = TorBoxProvider {
        api_key: prefs.torbox_api_key.clone(),
    };
    let real_debrid = RealDebridProvider {
        api_key: prefs.real_debrid_api_key.clone(),
    };

    let mut providers: Vec<&dyn SourceProvider> = Vec::new();
    if !prefs.torbox_api_key.is_empty() {
        providers.push(&torbox);
    }
    if !prefs.real_debrid_api_key.is_empty() {
        providers.push(&real_debrid);
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

    // 5. Convert top ranked results to playable streams for Stremio
    let mut streams = Vec::new();
    for entry in ranked.into_iter().filter(|r| r.score > 0).take(5) {
        // We already inject the correct /resolve/ URL during search!
        if let Some(direct_url) = entry.source.url.clone() {
            let explanation = entry
                .source
                .verification_reasons
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            streams.push(StremioStream {
                title: format!(
                    "🌟 Atlas | {} {} | Confidence {}%\n{}\n{}",
                    entry.source.provider_name,
                    entry.source.resolution,
                    entry.source.verification_score,
                    entry.source.title,
                    explanation
                ),
                url: direct_url,
            });
        }
    }

    streams
}
