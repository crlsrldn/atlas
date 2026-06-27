use crate::engines::identity::AtlasID;
use reqwest;
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct YTSTorrent {
    pub hash: String,
    pub quality: String,
    pub type_field: Option<String>,
    pub size_bytes: u64,
    pub video_codec: String,
    pub has_hdr: bool,
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

pub async fn get_metadata(atlas_id: &AtlasID) -> MediaMetadata {
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

            let normalized = fetch_cinemeta_metadata(id, media_type, *season, *episode).await;
            let torrentio_url = match (season, episode) {
                (Some(season), Some(episode)) => {
                    format!(
                        "https://torrentio.strem.fun/stream/series/{}:{}:{}.json",
                        id, season, episode
                    )
                }
                _ => format!("https://torrentio.strem.fun/stream/movie/{}.json", id),
            };

            let torrents = fetch_torrentio_sources(&torrentio_url)
                .await
                .unwrap_or_else(|| {
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
    let response = reqwest::get(url).await.ok()?;
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
    let res = reqwest::get(url).await.ok()?;
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

            torrents.push(YTSTorrent {
                hash,
                quality,
                type_field: None,
                size_bytes: parse_size_bytes(&title_str).unwrap_or(0),
                video_codec,
                has_hdr,
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

#[cfg(test)]
mod tests {
    use super::{parse_runtime_minutes, parse_size_bytes, parse_year};

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
}
