use crate::engines::metadata::MediaMetadata;
use crate::engines::sources::SourceResult;

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub confidence: u8,
    pub reasons: Vec<String>,
}

pub fn verify_source(source: &SourceResult, metadata: &MediaMetadata) -> VerificationResult {
    let mut score: i32 = 35;
    let mut reasons = Vec::new();
    let evidence = format!("{} {}", source.title, source.raw_title).to_lowercase();

    if source.hash.as_ref().is_some_and(|hash| hash.len() >= 32) {
        score += 15;
        reasons.push("✨ Verified Hash".to_string());
    }

    let title_tokens = title_tokens(&metadata.title);
    let matched_tokens = title_tokens
        .iter()
        .filter(|token| evidence.contains(token.as_str()))
        .count();
    if !title_tokens.is_empty() && matched_tokens >= title_tokens.len().min(2) {
        score += 20;
        reasons.push("🎯 Title Match".to_string());
    }

    if let Some(year) = metadata.year {
        if evidence.contains(&year.to_string()) {
            score += 10;
            reasons.push("📅 Year Match".to_string());
        }
    }

    if let (Some(season), Some(episode)) = (metadata.season, metadata.episode) {
        if episode_marker_matches(&evidence, season, episode) {
            score += 25;
            reasons.push(format!("📺 S{:02}E{:02} Match", season, episode));
        } else {
            score -= 30;
            reasons.push("⚠️ Missing Episode Marker".to_string());
        }
    }

    if source.is_cached {
        score += 10;
        reasons.push("⚡️ Instant Play".to_string());
    }

    if source.size_bytes.unwrap_or(0) > 100_000_000 {
        score += 5;
        reasons.push("📦 Valid Size".to_string());
    }

    if source.release_group.is_some() {
        score += 5;
        reasons.push("🏆 Trusted Group".to_string());
    }

    if metadata.runtime_minutes.is_some() {
        score += 5;
        reasons.push("⏳ Runtime Confirmed".to_string());
    }

    if has_language_evidence(&evidence) {
        score += 5;
        reasons.push("🗣️ Audio Match".to_string());
    }

    if has_file_structure_evidence(&evidence) {
        score += 5;
        reasons.push("📂 Valid Structure".to_string());
    }

    VerificationResult {
        confidence: score.clamp(0, 100) as u8,
        reasons,
    }
}

fn title_tokens(title: &str) -> Vec<String> {
    title
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| token.len() >= 4)
        .take(4)
        .map(|token| token.to_lowercase())
        .collect()
}

fn episode_marker_matches(evidence: &str, season: u32, episode: u32) -> bool {
    let compact = evidence.replace([' ', '.', '-', '_'], "");
    compact.contains(&format!("s{:02}e{:02}", season, episode))
        || compact.contains(&format!("{}x{:02}", season, episode))
        || compact.contains(&format!("season{}episode{}", season, episode))
}

fn has_language_evidence(evidence: &str) -> bool {
    [
        " english ",
        ".english.",
        ".eng.",
        " eng ",
        " multi ",
        " dual ",
        " dubbed ",
        " subbed ",
        " japanese ",
        " french ",
        " spanish ",
    ]
    .iter()
    .any(|marker| evidence.contains(marker))
}

fn has_file_structure_evidence(evidence: &str) -> bool {
    [".mkv", ".mp4", ".avi", ".webm", "2160p", "1080p", "720p"]
        .iter()
        .any(|marker| evidence.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::verify_source;
    use crate::engines::identity::AtlasID;
    use crate::engines::metadata::MediaMetadata;
    use crate::engines::sources::SourceResult;

    fn metadata() -> MediaMetadata {
        MediaMetadata {
            atlas_id: AtlasID::IMDb {
                id: "tt0944947".to_string(),
                season: Some(1),
                episode: Some(2),
            },
            imdb_id: Some("tt0944947".to_string()),
            title: "Game of Thrones - The Kingsroad".to_string(),
            year: Some(2011),
            media_type: "series".to_string(),
            season: Some(1),
            episode: Some(2),
            runtime_minutes: Some(56),
            genres: vec!["Drama".to_string()],
            release_date: Some("2011-04-24".to_string()),
            torrents: vec![],
        }
    }

    fn source(raw_title: &str) -> SourceResult {
        SourceResult {
            provider_name: "Test".to_string(),
            provider_priority: 90,
            provider_latency_ms: Some(100),
            title: raw_title.to_string(),
            raw_title: raw_title.to_string(),
            hash: Some("1234567890abcdef1234567890abcdef".to_string()),
            size_bytes: Some(1_000_000_000),
            bitrate_mbps: Some(12.0),
            resolution: "1080p".to_string(),
            codec: "HEVC".to_string(),
            audio_codec: Some("AAC".to_string()),
            audio_channels: Some("2.0".to_string()),
            has_hdr: false,
            has_dolby_vision: false,
            has_subtitles: true,
            is_cached: true,
            url: Some("http://example.test".to_string()),
            release_group: Some("GROUP".to_string()),
            verification_score: 0,
            verification_reasons: vec![],
            playback_successes: 0,
            playback_failures: 0,
        }
    }

    #[test]
    fn rewards_episode_match_evidence() {
        let result = verify_source(
            &source("Game.of.Thrones.S01E02.2011.1080p.English.mkv-GROUP"),
            &metadata(),
        );

        assert!(result.confidence >= 90);
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.contains("S01E02")));
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason == "🗣️ Audio Match"));
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason == "📂 Valid Structure"));
    }

    #[test]
    fn penalizes_missing_episode_marker() {
        let result = verify_source(
            &source("Game.of.Thrones.S01E03.2011.1080p-GROUP"),
            &metadata(),
        );

        assert!(result.confidence < 90);
    }
}
