use crate::engines::sources::SourceResult;
use crate::api::config::UserPreferences;

#[derive(Debug, Clone)]
pub struct RankedSource {
    pub source: SourceResult,
    pub score: u64,
}

pub fn rank_sources(sources: Vec<SourceResult>, prefs: &UserPreferences) -> Vec<RankedSource> {
    let mut ranked: Vec<RankedSource> = sources.into_iter().map(|source| {
        let score = calculate_score(&source, prefs);
        RankedSource { source, score }
    }).collect();

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
    } else if prefs.max_resolution == "720p" && (source.resolution == "4K" || source.resolution == "1080p") {
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
