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

#[cfg(test)]
mod tests {
    use super::AtlasID;

    #[test]
    fn parses_movie_imdb_id() {
        let parsed = AtlasID::from_stremio_id("tt0133093").expect("valid IMDb ID");

        match parsed {
            AtlasID::IMDb {
                id,
                season,
                episode,
            } => {
                assert_eq!(id, "tt0133093");
                assert_eq!(season, None);
                assert_eq!(episode, None);
            }
            _ => panic!("expected IMDb ID"),
        }
    }

    #[test]
    fn preserves_series_episode_context() {
        let parsed = AtlasID::from_stremio_id("tt0944947:1:2").expect("valid series ID");

        match parsed {
            AtlasID::IMDb {
                id,
                season,
                episode,
            } => {
                assert_eq!(id, "tt0944947");
                assert_eq!(season, Some(1));
                assert_eq!(episode, Some(2));
            }
            _ => panic!("expected IMDb ID"),
        }
    }

    #[test]
    fn parses_tmdb_id() {
        let parsed = AtlasID::from_stremio_id("tmdb:550").expect("valid TMDB ID");

        match parsed {
            AtlasID::TMDB(id) => assert_eq!(id, 550),
            _ => panic!("expected TMDB ID"),
        }
    }
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

    pub fn season_episode(&self) -> Option<(u32, u32)> {
        match self {
            AtlasID::IMDb {
                season: Some(season),
                episode: Some(episode),
                ..
            } => Some((*season, *episode)),
            _ => None,
        }
    }

    pub fn media_type(&self) -> &'static str {
        if self.season_episode().is_some() {
            "series"
        } else {
            "movie"
        }
    }
}
