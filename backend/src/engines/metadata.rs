use crate::engines::identity::AtlasID;
use reqwest;
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct MediaMetadata {
    pub title: String,
    pub year: u32,
    pub media_type: String, // "movie" or "series"
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
}

#[derive(Deserialize)]
struct TorrentioResponse {
    streams: Option<Vec<TorrentioStream>>,
}

#[derive(Deserialize)]
struct TorrentioStream {
    name: Option<String>,  // e.g. "Torrentio\n4k HDR"
    title: Option<String>, // e.g. "Interstellar... \n 16 GB"
    #[serde(rename = "infoHash")]
    info_hash: Option<String>,
}

pub async fn get_metadata(atlas_id: &AtlasID) -> MediaMetadata {
    match atlas_id {
        AtlasID::IMDb {
            id,
            season,
            episode,
        } => {
            // Using Torrentio as a fallback indexer since YTS is blocked
            let (url, media_type) = match (season, episode) {
                (Some(season), Some(episode)) => (
                    format!(
                        "https://torrentio.strem.fun/stream/series/{}:{}:{}.json",
                        id, season, episode
                    ),
                    "series",
                ),
                _ => (
                    format!("https://torrentio.strem.fun/stream/movie/{}.json", id),
                    "movie",
                ),
            };

            if let Ok(res) = reqwest::get(&url).await {
                if let Ok(json) = res.json::<TorrentioResponse>().await {
                    if let Some(streams) = json.streams {
                        let mut torrents = Vec::new();
                        for stream in streams {
                            if let Some(hash) = stream.info_hash {
                                let name = stream.name.unwrap_or_default().to_lowercase();

                                let quality = if name.contains("4k") {
                                    "4K".to_string()
                                } else if name.contains("1080p") {
                                    "1080p".to_string()
                                } else {
                                    "720p".to_string()
                                };

                                let title_str = stream.title.unwrap_or_default();
                                let video_codec = if title_str.to_lowercase().contains("av1") {
                                    "AV1".to_string()
                                } else if title_str.to_lowercase().contains("hevc")
                                    || title_str.to_lowercase().contains("x265")
                                {
                                    "HEVC".to_string()
                                } else {
                                    "H264".to_string()
                                };

                                let has_hdr = name.contains("hdr")
                                    || title_str.to_lowercase().contains("hdr")
                                    || title_str.to_lowercase().contains("dv")
                                    || title_str.to_lowercase().contains("vision");

                                torrents.push(YTSTorrent {
                                    hash,
                                    quality,
                                    type_field: None,
                                    size_bytes: 5_000_000_000, // mock size for MVP
                                    video_codec,
                                    has_hdr,
                                });

                                if torrents.len() >= 100 {
                                    // Fetch 100 to ensure 1080p and SDR are included
                                    break;
                                }
                            }
                        }
                        return MediaMetadata {
                            title: format!("Movie ({})", id),
                            year: 0,
                            media_type: media_type.to_string(),
                            torrents,
                        };
                    }
                }
            }

            warn!("Torrentio fetch failed or returned no results for {}", id);
            MediaMetadata {
                title: format!("Movie ({})", id),
                year: 0,
                media_type: media_type.to_string(),
                torrents: vec![],
            }
        }
        AtlasID::TMDB(id) => MediaMetadata {
            title: format!("TMDB Series ({})", id),
            year: 0,
            media_type: "series".to_string(),
            torrents: vec![],
        },
    }
}
