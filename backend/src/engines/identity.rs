use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtlasID {
    IMDb {
        id: String,
        season: Option<u32>,
        episode: Option<u32>,
    },
    TMDB(u32),
    // ... TVDB, Trakt, AniDB to be added later
}

impl AtlasID {
    /// Normalize a Stremio ID into an AtlasID.
    /// Stremio IDs are typically IMDb IDs (e.g., "tt1234567") or Kitsu IDs for anime.
    pub fn from_stremio_id(id: &str) -> Option<Self> {
        if id.starts_with("tt") {
            let mut parts = id.split(':');
            let imdb_id = parts.next()?.to_string();
            let season = parts.next().and_then(|value| value.parse::<u32>().ok());
            let episode = parts.next().and_then(|value| value.parse::<u32>().ok());
            Some(AtlasID::IMDb {
                id: imdb_id,
                season,
                episode,
            })
        } else if id.starts_with("tmdb:") {
            let num = id.trim_start_matches("tmdb:").parse::<u32>().ok()?;
            Some(AtlasID::TMDB(num))
        } else {
            None
        }
    }
}
