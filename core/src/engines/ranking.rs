use crate::api::config::UserPreferences;
use crate::engines::sources::SourceResult;

#[derive(Debug, Clone)]
pub struct RankedSource {
    pub source: SourceResult,
    pub score: u64,
}

pub fn rank_sources(
    sources: Vec<SourceResult>,
    prefs: &UserPreferences,
    monetization_enabled: bool,
) -> Vec<RankedSource> {
    let mut ranked: Vec<RankedSource> = sources
        .into_iter()
        .map(|source| {
            let score = calculate_score(&source, prefs, monetization_enabled);
            RankedSource { source, score }
        })
        .collect();

    // Sort descending by score
    ranked.sort_by_key(|entry| std::cmp::Reverse(entry.score));

    ranked
}

fn calculate_score(
    source: &SourceResult,
    prefs: &UserPreferences,
    monetization_enabled: bool,
) -> u64 {
    let mut score: u64 = 1000;

    // Premium Restrictions
    if monetization_enabled && !prefs.is_premium {
        if !source.is_cached {
            return 0; // Only cached streams for free users
        }
        if is_above_max_resolution(&source.resolution, "1080p") {
            return 0; // 1080p max for free users
        }
    }

    // Compatibility and Codecs
    if prefs.exclude_av1 && source.codec == "AV1" {
        return 0; // completely exclude
    }

    // Quality Matching
    if source.resolution == prefs.max_resolution {
        score += 500;
    } else if is_above_max_resolution(&source.resolution, &prefs.max_resolution) {
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
        Some(_) => {}                             // known other
        None => score = score.saturating_sub(50), // penalty for missing audio codec
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
    } else if let Some(size_bytes) = source.size_bytes {
        // Fallback to raw size if bitrate is unknown (e.g. unknown runtime)
        let gb = size_bytes as f64 / 1_073_741_824.0;
        if gb >= 15.0 {
            score += 150;
        } else if gb >= 5.0 {
            score += 90;
        } else if gb >= 1.5 {
            score += 40;
        }
    }

    match source.codec.as_str() {
        "HEVC" | "H265" => score += 60, // Boost modern efficient codecs
        _ => {}
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

    // Apply Sorting Preferences
    match prefs.sort_preference.as_str() {
        "quality" => {
            if let Some(size) = source.size_bytes {
                let gb = size as f64 / 1_073_741_824.0;
                score += (gb * 100.0) as u64; // Huge boost to larger files
            } else if let Some(mbps) = source.bitrate_mbps {
                score += (mbps as f64 * 10.0) as u64;
            }
        }
        "speed" => {
            score += 10000; // Elevate all streams to prevent penalty from dropping below 0 easily
            if let Some(size) = source.size_bytes {
                let gb = size as f64 / 1_073_741_824.0;
                let penalty = (gb * 100.0) as u64;
                score = score.saturating_sub(penalty); // Heavily penalize large files
            }
        }
        _ => {} // "balanced"
    }

    // Availability (Cached only heavily prioritized based on PRD)
    if !source.is_cached {
        score /= 10;
    }

    score
}

fn is_above_max_resolution(source_resolution: &str, max_resolution: &str) -> bool {
    matches!(
        (source_resolution, max_resolution),
        ("4K", "1080p") | ("4K", "720p") | ("1080p", "720p")
    )
}

#[cfg(test)]
mod tests {
    use super::rank_sources;
    use crate::api::config::UserPreferences;
    use crate::engines::sources::SourceResult;

    fn prefs() -> UserPreferences {
        UserPreferences {
            max_resolution: "1080p".to_string(),
            exclude_av1: true,
            ..UserPreferences::default()
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
        let ranked = rank_sources(vec![source("1080p", "AV1", false, true)], &prefs(), false);

        assert_eq!(ranked[0].score, 0);
    }

    #[test]
    fn excludes_resolution_above_user_maximum() {
        let ranked = rank_sources(vec![source("4K", "HEVC", false, true)], &prefs(), false);

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
            false,
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

        let ranked = rank_sources(vec![slow, fast], &prefs(), false);

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

        let ranked = rank_sources(vec![failed, successful], &prefs(), false);

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

        let ranked = rank_sources(vec![basic, rich], &prefs(), false);

        assert_eq!(ranked[0].source.provider_name, "Rich");
    }
}
#[cfg(test)]
mod tests2 {
    use super::*;

    fn mock_source() -> SourceResult {
        SourceResult {
            provider_name: "Mock".to_string(),
            provider_priority: 50,
            provider_latency_ms: Some(200),
            title: "Movie".to_string(),
            raw_title: "Movie".to_string(),
            hash: Some("abc".to_string()),
            size_bytes: Some(1_000_000_000), // 1GB
            bitrate_mbps: None,
            resolution: "1080p".to_string(),
            codec: "H264".to_string(),
            audio_codec: Some("AAC".to_string()),
            audio_channels: Some("2.0".to_string()),
            has_hdr: false,
            has_dolby_vision: false,
            has_subtitles: false,
            is_cached: true,
            url: None,
            release_group: None,
            verification_score: 100,
            verification_reasons: vec![],
            playback_successes: 0,
            playback_failures: 0,
        }
    }

    fn mock_prefs() -> UserPreferences {
        UserPreferences {
            torbox_api_key: "".to_string(),

            gemini_api_key: "".to_string(),
            trakt_client_id: "".to_string(),
            trakt_username: "".to_string(),
            max_resolution: "4K".to_string(),
            prefer_hdr: true,
            exclude_av1: false,
            exclude_hevc: false,
            profile: "default".to_string(),
            mobile_data_saver: false,
            home_theater_mode: false,
            family_mode: false,
            preferred_language: "en".to_string(),
            subtitle_mode: "auto".to_string(),
            sort_preference: "balanced".to_string(),
            stream_limit: 5,
            is_premium: false,
        }
    }

    #[test]
    fn test_hevc_boost() {
        let prefs = mock_prefs();
        let mut h264 = mock_source();
        h264.codec = "H264".to_string();

        let mut hevc = mock_source();
        hevc.codec = "HEVC".to_string();

        assert!(calculate_score(&hevc, &prefs, false) > calculate_score(&h264, &prefs, false));
    }

    #[test]
    fn test_size_fallback_boost() {
        let prefs = mock_prefs();

        let mut small = mock_source();
        small.size_bytes = Some(1_000_000_000); // 1GB (no size boost)

        let mut large = mock_source();
        large.size_bytes = Some(20_000_000_000); // 20GB (should get 150 boost)

        assert!(calculate_score(&large, &prefs, false) > calculate_score(&small, &prefs, false));
    }

    #[test]
    fn test_sort_preference() {
        let mut prefs = mock_prefs();

        let mut small = mock_source();
        small.size_bytes = Some(1_000_000_000); // 1GB

        let mut large = mock_source();
        large.size_bytes = Some(20_000_000_000); // 20GB

        // Default (balanced) ranks large higher due to size boost
        assert!(calculate_score(&large, &prefs, false) > calculate_score(&small, &prefs, false));

        // Speed prefers smaller files
        prefs.sort_preference = "speed".to_string();
        assert!(calculate_score(&small, &prefs, false) > calculate_score(&large, &prefs, false));

        // Quality prefers larger files (more aggressively)
        prefs.sort_preference = "quality".to_string();
        assert!(calculate_score(&large, &prefs, false) > calculate_score(&small, &prefs, false));
    }
}
