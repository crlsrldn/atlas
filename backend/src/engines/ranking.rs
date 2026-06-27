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
        score = 0;
    } else if prefs.max_resolution == "720p"
        && (source.resolution == "4K" || source.resolution == "1080p")
    {
        score = 0;
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

    // Compatibility
    if prefs.exclude_av1 && source.codec == "AV1" {
        score = 0; // completely exclude
    }

    // Provider Priority could be injected here, but for now we just use the raw score.

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
            title: "Test Source".to_string(),
            hash: Some("abc".to_string()),
            size_bytes: Some(1),
            resolution: resolution.to_string(),
            codec: codec.to_string(),
            has_hdr,
            is_cached,
            url: Some("http://example.test".to_string()),
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
}
