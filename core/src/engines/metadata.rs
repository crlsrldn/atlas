use crate::engines::cache::{get_json, set_json, METADATA_TTL};
use crate::engines::identity::AtlasID;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub atlas_id: AtlasID,
    pub imdb_id: Option<String>,
    pub title: String,
    pub year: Option<u32>,
    pub media_type: String,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub runtime_minutes: Option<u32>,
    pub genres: Vec<String>,
    pub release_date: Option<String>,
    pub torrents: Vec<YTSTorrent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YTSTorrent {
    pub hash: String,
    pub quality: String,
    pub type_field: Option<String>,
    pub size_bytes: u64,
    pub video_codec: String,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<String>,
    pub bitrate_mbps: Option<f32>,
    pub has_hdr: bool,
    pub has_dolby_vision: bool,
    pub has_subtitles: bool,
    pub raw_title: String,
    pub release_group: Option<String>,
}

#[derive(Debug, Clone)]
struct NormalizedMetadata {
    title: Option<String>,
    year: Option<u32>,
    runtime_minutes: Option<u32>,
    genres: Vec<String>,
    release_date: Option<String>,
}

#[derive(Deserialize)]
struct TorrentioResponse {
    streams: Option<Vec<TorrentioStream>>,
}

#[derive(Deserialize)]
struct TorrentioStream {
    name: Option<String>,
    title: Option<String>,
    #[serde(rename = "infoHash")]
    info_hash: Option<String>,
}

#[derive(Deserialize)]
struct CinemetaResponse {
    meta: Option<CinemetaMeta>,
}

#[derive(Deserialize)]
struct CinemetaMeta {
    name: Option<String>,
    #[serde(rename = "releaseInfo")]
    release_info: Option<String>,
    runtime: Option<String>,
    genres: Option<Vec<String>>,
    released: Option<String>,
    #[serde(default)]
    videos: Vec<CinemetaVideo>,
}

#[derive(Deserialize)]
struct CinemetaVideo {
    season: Option<u32>,
    episode: Option<u32>,
    title: Option<String>,
    released: Option<String>,
    runtime: Option<String>,
}

/// Metadata is identical for every user, so it is cached process-wide.
/// Without this, each stream request re-fetches Cinemeta and Torrentio.
pub async fn get_metadata(atlas_id: &AtlasID) -> MediaMetadata {
    let cache_key = metadata_cache_key(atlas_id);

    if let Some(cached) = get_json(&cache_key) {
        if let Ok(metadata) = serde_json::from_value::<MediaMetadata>(cached) {
            return metadata;
        }
    }

    let metadata = fetch_metadata(atlas_id).await;

    // Only cache a useful answer: an upstream blip would otherwise pin an
    // empty source list in front of this title for a day.
    if !metadata.torrents.is_empty() {
        if let Ok(value) = serde_json::to_value(&metadata) {
            set_json(cache_key, value, METADATA_TTL);
        }
    }

    metadata
}

fn metadata_cache_key(atlas_id: &AtlasID) -> String {
    match atlas_id {
        AtlasID::IMDb {
            id,
            season: Some(season),
            episode: Some(episode),
        } => format!("metadata:{}:{}:{}", id, season, episode),
        AtlasID::IMDb { id, .. } => format!("metadata:{}", id),
        AtlasID::TMDB(id) => format!("metadata:tmdb:{}", id),
    }
}

async fn fetch_metadata(atlas_id: &AtlasID) -> MediaMetadata {
    match atlas_id {
        AtlasID::IMDb {
            id,
            season,
            episode,
        } => {
            let media_type = if season.is_some() && episode.is_some() {
                "series"
            } else {
                "movie"
            };

            let torrentio_url = match (season, episode) {
                (Some(season), Some(episode)) => {
                    format!(
                        "https://torrentio.strem.fun/stream/series/{}:{}:{}.json",
                        id, season, episode
                    )
                }
                _ => format!("https://torrentio.strem.fun/stream/movie/{}.json", id),
            };

            // Independent upstreams — fetch them at the same time rather than
            // paying both round trips back to back.
            let (normalized, torrents) = tokio::join!(
                fetch_cinemeta_metadata(id, media_type, *season, *episode),
                fetch_torrentio_sources(&torrentio_url)
            );

            let torrents = torrents.unwrap_or_else(|| {
                warn!("Torrentio fetch failed or returned no results for {}", id);
                vec![]
            });

            MediaMetadata {
                atlas_id: atlas_id.clone(),
                imdb_id: Some(id.clone()),
                title: normalized
                    .as_ref()
                    .and_then(|meta| meta.title.clone())
                    .unwrap_or_else(|| format!("IMDb {}", id)),
                year: normalized.as_ref().and_then(|meta| meta.year),
                media_type: media_type.to_string(),
                season: *season,
                episode: *episode,
                runtime_minutes: normalized.as_ref().and_then(|meta| meta.runtime_minutes),
                genres: normalized
                    .as_ref()
                    .map(|meta| meta.genres.clone())
                    .unwrap_or_default(),
                release_date: normalized.and_then(|meta| meta.release_date),
                torrents,
            }
        }
        AtlasID::TMDB(id) => MediaMetadata {
            atlas_id: atlas_id.clone(),
            imdb_id: None,
            title: format!("TMDB {}", id),
            year: None,
            media_type: "series".to_string(),
            season: None,
            episode: None,
            runtime_minutes: None,
            genres: vec![],
            release_date: None,
            torrents: vec![],
        },
    }
}

async fn fetch_cinemeta_metadata(
    imdb_id: &str,
    media_type: &str,
    season: Option<u32>,
    episode: Option<u32>,
) -> Option<NormalizedMetadata> {
    let url = format!(
        "https://v3-cinemeta.strem.io/meta/{}/{}.json",
        media_type, imdb_id
    );
    let response = crate::engines::http::client().get(url).send().await.ok()?;
    let body = response.json::<CinemetaResponse>().await.ok()?;
    let meta = body.meta?;

    let matching_video = meta.videos.iter().find(|video| {
        season.is_some() && episode.is_some() && video.season == season && video.episode == episode
    });

    let title = match matching_video.and_then(|video| video.title.clone()) {
        Some(episode_title) => meta
            .name
            .as_ref()
            .map(|series_title| format!("{} - {}", series_title, episode_title))
            .or(Some(episode_title)),
        None => meta.name,
    };

    let release_date = matching_video
        .and_then(|video| video.released.clone())
        .or(meta.released);

    let runtime_minutes = matching_video
        .and_then(|video| video.runtime.as_deref().and_then(parse_runtime_minutes))
        .or_else(|| meta.runtime.as_deref().and_then(parse_runtime_minutes));

    Some(NormalizedMetadata {
        title,
        year: meta.release_info.as_deref().and_then(parse_year),
        runtime_minutes,
        genres: meta.genres.unwrap_or_default(),
        release_date,
    })
}

async fn fetch_torrentio_sources(url: &str) -> Option<Vec<YTSTorrent>> {
    let res = crate::engines::http::client().get(url).send().await.ok()?;
    let json = res.json::<TorrentioResponse>().await.ok()?;
    let streams = json.streams?;
    let mut torrents = Vec::new();

    for stream in streams {
        if let Some(hash) = stream.info_hash {
            let name = stream.name.unwrap_or_default().to_lowercase();
            let title_str = stream.title.unwrap_or_default();
            let title_lower = title_str.to_lowercase();

            let quality = if name.contains("4k") || title_lower.contains("2160p") {
                "4K".to_string()
            } else if name.contains("1080p") || title_lower.contains("1080p") {
                "1080p".to_string()
            } else {
                "720p".to_string()
            };

            let video_codec = if title_lower.contains("av1") {
                "AV1".to_string()
            } else if title_lower.contains("hevc") || title_lower.contains("x265") {
                "HEVC".to_string()
            } else {
                "H264".to_string()
            };

            let has_hdr = name.contains("hdr")
                || title_lower.contains("hdr")
                || title_lower.contains("dv")
                || title_lower.contains("vision");
            let has_dolby_vision = title_lower.contains(" dolby vision")
                || title_lower.contains(" dovi")
                || title_lower.contains(".dv.")
                || title_lower.contains(" dv ");
            let audio_codec = infer_audio_codec(&title_lower);
            let audio_channels = infer_audio_channels(&title_lower);
            let size_bytes = parse_size_bytes(&title_str).unwrap_or(0);

            torrents.push(YTSTorrent {
                hash,
                quality,
                type_field: None,
                size_bytes,
                video_codec,
                audio_codec,
                audio_channels,
                bitrate_mbps: estimate_bitrate_mbps(size_bytes, None),
                has_hdr,
                has_dolby_vision,
                has_subtitles: has_subtitle_evidence(&title_lower),
                raw_title: title_str.clone(),
                release_group: infer_release_group(&title_str),
            });

            if torrents.len() >= 100 {
                break;
            }
        }
    }

    Some(torrents)
}

fn parse_runtime_minutes(runtime: &str) -> Option<u32> {
    let digits: String = runtime.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn parse_year(value: &str) -> Option<u32> {
    value
        .split(|c: char| !c.is_ascii_digit())
        .find(|part| part.len() == 4)
        .and_then(|part| part.parse().ok())
}

fn parse_size_bytes(value: &str) -> Option<u64> {
    let lower = value.to_lowercase();
    for unit in ["gb", "mb"] {
        if let Some(index) = lower.find(unit) {
            let before = &lower[..index];
            let number = before
                .split_whitespace()
                .last()
                .and_then(|part| part.parse::<f64>().ok())?;
            let multiplier = if unit == "gb" {
                1_000_000_000.0
            } else {
                1_000_000.0
            };
            return Some((number * multiplier) as u64);
        }
    }

    None
}

fn infer_release_group(title: &str) -> Option<String> {
    title
        .rsplit_once('-')
        .map(|(_, group)| group.trim())
        .filter(|group| !group.is_empty() && group.len() <= 24)
        .map(ToString::to_string)
}

fn infer_audio_codec(value: &str) -> Option<String> {
    if value.contains("truehd") {
        Some("TrueHD".to_string())
    } else if value.contains("atmos") {
        Some("Atmos".to_string())
    } else if value.contains("dts") {
        Some("DTS".to_string())
    } else if value.contains("aac") {
        Some("AAC".to_string())
    } else if value.contains("ac3") || value.contains("ddp") || value.contains("eac3") {
        Some("Dolby Digital".to_string())
    } else {
        None
    }
}

fn infer_audio_channels(value: &str) -> Option<String> {
    if value.contains("7.1") {
        Some("7.1".to_string())
    } else if value.contains("5.1") {
        Some("5.1".to_string())
    } else if value.contains("2.0") {
        Some("2.0".to_string())
    } else {
        None
    }
}

fn has_subtitle_evidence(value: &str) -> bool {
    value.contains("sub")
        || value.contains("multi")
        || value.contains("vost")
        || value.contains("cc")
}

fn estimate_bitrate_mbps(size_bytes: u64, runtime_minutes: Option<u32>) -> Option<f32> {
    let runtime_minutes = runtime_minutes.unwrap_or(100);
    if size_bytes == 0 || runtime_minutes == 0 {
        return None;
    }
    let megabits = (size_bytes as f32 * 8.0) / 1_000_000.0;
    Some(megabits / (runtime_minutes as f32 * 60.0))
}

#[cfg(test)]
mod tests {
    use super::{
        infer_audio_codec, metadata_cache_key, parse_runtime_minutes, parse_size_bytes, parse_year,
    };
    use crate::engines::identity::AtlasID;

    #[test]
    fn metadata_cache_keys_separate_episodes_and_titles() {
        let movie = AtlasID::IMDb {
            id: "tt0133093".to_string(),
            season: None,
            episode: None,
        };
        let s1e1 = AtlasID::IMDb {
            id: "tt0903747".to_string(),
            season: Some(1),
            episode: Some(1),
        };
        let s1e2 = AtlasID::IMDb {
            id: "tt0903747".to_string(),
            season: Some(1),
            episode: Some(2),
        };

        assert_eq!(metadata_cache_key(&movie), "metadata:tt0133093");
        assert_ne!(metadata_cache_key(&s1e1), metadata_cache_key(&s1e2));
        assert_ne!(metadata_cache_key(&movie), metadata_cache_key(&s1e1));
        assert_eq!(metadata_cache_key(&AtlasID::TMDB(603)), "metadata:tmdb:603");
    }

    #[test]
    fn parses_cinemeta_year_and_runtime() {
        assert_eq!(parse_year("2014-2019"), Some(2014));
        assert_eq!(parse_runtime_minutes("49 min"), Some(49));
    }

    #[test]
    fn parses_torrentio_size_text() {
        assert_eq!(parse_size_bytes("Movie\n1.5 GB"), Some(1_500_000_000));
        assert_eq!(parse_size_bytes("Episode\n700 MB"), Some(700_000_000));
    }

    #[test]
    fn infers_audio_codec() {
        assert_eq!(
            infer_audio_codec("movie truehd atmos 7.1"),
            Some("TrueHD".to_string())
        );
        assert_eq!(infer_audio_codec("movie aac 2.0"), Some("AAC".to_string()));
    }
}
