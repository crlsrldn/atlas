use crate::api::config::UserPreferences;
use crate::engines::sources::SourceResult;

#[derive(Debug, Clone)]
pub struct RankedSource {
    pub source: SourceResult,
    pub score: u64,
}

pub fn rank_sources(sources: Vec<SourceResult>, prefs: &UserPreferences) -> Vec<RankedSource> {
    let mut ranked: Vec<RankedSource> = sources
        .into_iter()
        .map(|source| {
            let score = calculate_score(&source, prefs);
            RankedSource { source, score }
        })
        .collect();

    // Sort descending by score
    ranked.sort_by(|a, b| b.score.cmp(&a.score));

    ranked
}

fn calculate_score(source: &SourceResult, prefs: &UserPreferences) -> u64 {
    let mut score: u64 = 1000;

    // Availability (Cached only heavily prioritized based on PRD)
    if !source.is_cached {
        score /= 10;
    }

    // Quality Matching
    if source.resolution == prefs.max_resolution {
        score += 500;
    } else if prefs.max_resolution == "1080p" && source.resolution == "4K" {
        return 0;
    } else if prefs.max_resolution == "720p"
        && (source.resolution == "4K" || source.resolution == "1080p")
    {
        return 0;
    }

    if source.has_hdr {
        if prefs.prefer_hdr {
            score += 300;
        } else {
            // Strongly demote HDR if the user disabled the "Prefer HDR" toggle
            // This prevents "too dark" issues on SDR displays
            score /= 10;
        }
    }

    if source.has_dolby_vision && prefs.prefer_hdr {
        score += 120;
    }

    if source.has_subtitles {
        score += 40;
    }

    match source.audio_codec.as_deref() {
        Some("TrueHD") | Some("Atmos") => score += 140,
        Some("DTS") => score += 100,
        Some("Dolby Digital") => score += 70,
        Some("AAC") => score += 30,
        _ => {}
    }

    match source.audio_channels.as_deref() {
        Some("7.1") => score += 80,
        Some("5.1") => score += 50,
        Some("2.0") => score += 10,
        _ => {}
    }

    if let Some(bitrate_mbps) = source.bitrate_mbps {
        if bitrate_mbps >= 45.0 {
            score += 160;
        } else if bitrate_mbps >= 18.0 {
            score += 110;
        } else if bitrate_mbps >= 6.0 {
            score += 60;
        } else {
            score /= 2;
        }
    }

    // Compatibility
    if prefs.exclude_av1 && source.codec == "AV1" {
        return 0; // completely exclude
    }

    score += u64::from(source.provider_priority) * 4;

    if let Some(latency_ms) = source.provider_latency_ms {
        if latency_ms <= 250 {
            score += 200;
        } else if latency_ms <= 1000 {
            score += 100;
        } else if latency_ms > 3000 {
            score /= 2;
        }
    }

    score += u64::from(source.verification_score) * 5;
    if source.verification_score < 45 {
        score /= 2;
    }

    score += u64::from(source.playback_successes) * 180;
    if source.playback_failures > 0 {
        let penalty = u64::from(source.playback_failures).saturating_mul(240);
        score = score.saturating_sub(penalty);
    }

    score
}

#[cfg(test)]
mod tests {
    use super::rank_sources;
    use crate::api::config::UserPreferences;
    use crate::engines::sources::SourceResult;

    fn prefs() -> UserPreferences {
        UserPreferences {
            torbox_api_key: String::new(),
            real_debrid_api_key: String::new(),
            gemini_api_key: String::new(),
            max_resolution: "1080p".to_string(),
            prefer_hdr: true,
            exclude_av1: true,
        }
    }

    fn source(resolution: &str, codec: &str, has_hdr: bool, is_cached: bool) -> SourceResult {
        SourceResult {
            provider_name: "Test".to_string(),
            provider_priority: 80,
            provider_latency_ms: Some(250),
            title: "Test Source".to_string(),
            raw_title: "Test Source".to_string(),
            hash: Some("abc".to_string()),
            size_bytes: Some(1),
            bitrate_mbps: Some(12.0),
            resolution: resolution.to_string(),
            codec: codec.to_string(),
            audio_codec: Some("AAC".to_string()),
            audio_channels: Some("2.0".to_string()),
            has_hdr,
            has_dolby_vision: false,
            has_subtitles: true,
            is_cached,
            url: Some("http://example.test".to_string()),
            release_group: Some("GROUP".to_string()),
            verification_score: 80,
            verification_reasons: vec!["test".to_string()],
            playback_successes: 0,
            playback_failures: 0,
        }
    }

    #[test]
    fn excludes_av1_when_preference_is_enabled() {
        let ranked = rank_sources(vec![source("1080p", "AV1", false, true)], &prefs());

        assert_eq!(ranked[0].score, 0);
    }

    #[test]
    fn excludes_resolution_above_user_maximum() {
        let ranked = rank_sources(vec![source("4K", "HEVC", false, true)], &prefs());

        assert_eq!(ranked[0].score, 0);
    }

    #[test]
    fn demotes_uncached_sources() {
        let ranked = rank_sources(
            vec![
                source("1080p", "HEVC", false, false),
                source("1080p", "HEVC", false, true),
            ],
            &prefs(),
        );

        assert!(ranked[0].source.is_cached);
        assert!(ranked[0].score > ranked[1].score);
    }

    #[test]
    fn prefers_healthier_provider_when_quality_matches() {
        let mut slow = source("1080p", "HEVC", false, true);
        slow.provider_name = "Slow".to_string();
        slow.provider_priority = 50;
        slow.provider_latency_ms = Some(4_000);

        let mut fast = source("1080p", "HEVC", false, true);
        fast.provider_name = "Fast".to_string();
        fast.provider_priority = 95;
        fast.provider_latency_ms = Some(100);

        let ranked = rank_sources(vec![slow, fast], &prefs());

        assert_eq!(ranked[0].source.provider_name, "Fast");
    }

    #[test]
    fn playback_history_feeds_ranking() {
        let mut failed = source("1080p", "HEVC", false, true);
        failed.provider_name = "Failed".to_string();
        failed.playback_failures = 3;

        let mut successful = source("1080p", "HEVC", false, true);
        successful.provider_name = "Successful".to_string();
        successful.playback_successes = 2;

        let ranked = rank_sources(vec![failed, successful], &prefs());

        assert_eq!(ranked[0].source.provider_name, "Successful");
    }

    #[test]
    fn richer_av_signals_improve_score() {
        let basic = source("1080p", "HEVC", false, true);
        let mut rich = source("1080p", "HEVC", true, true);
        rich.provider_name = "Rich".to_string();
        rich.audio_codec = Some("TrueHD".to_string());
        rich.audio_channels = Some("7.1".to_string());
        rich.bitrate_mbps = Some(50.0);
        rich.has_dolby_vision = true;

        let ranked = rank_sources(vec![basic, rich], &prefs());

        assert_eq!(ranked[0].source.provider_name, "Rich");
    }
}
