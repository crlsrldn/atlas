//! Turning catalogue entries into Jellyfin items.

use crate::api::jellyfin::dto::{BaseItemDto, UserItemDataDto};
use crate::api::jellyfin::ids::{ItemId, Library, Namespace};
use crate::engines::catalog::{CatalogEntry, EpisodeMeta};

/// Jellyfin measures time in 100-nanosecond ticks. Getting this wrong does not
/// fail loudly — it silently breaks every scrubber and resume point.
const TICKS_PER_MINUTE: i64 = 60 * 10_000_000;

pub fn ticks_from_minutes(minutes: u32) -> i64 {
    i64::from(minutes) * TICKS_PER_MINUTE
}

fn imdb_number(imdb_id: &str) -> u64 {
    imdb_id
        .trim_start_matches("tt")
        .parse::<u64>()
        .unwrap_or_default()
}

/// Clients fetch artwork only when an item advertises a tag for it. The value
/// is an opaque cache key, so the stable id serves.
fn image_tags(imdb_id: &str) -> (std::collections::HashMap<String, String>, Vec<String>) {
    let mut primary = std::collections::HashMap::new();
    primary.insert("Primary".to_string(), imdb_id.to_string());
    (primary, vec![imdb_id.to_string()])
}

fn base(id: String, name: String, server: String, imdb_id: &str) -> BaseItemDto {
    let mut item = BaseItemDto::folder(id, name, server);
    let (image_tags, backdrops) = image_tags(imdb_id);
    item.image_tags = image_tags;
    item.backdrop_image_tags = backdrops;
    item.provider_ids
        .insert("Imdb".to_string(), imdb_id.to_string());
    item.user_data = Some(UserItemDataDto {
        key: imdb_id.to_string(),
        ..UserItemDataDto::default()
    });
    item
}

fn apply_entry(item: &mut BaseItemDto, entry: &CatalogEntry) {
    item.overview = entry.description.clone();
    item.genres = entry.genres.clone();
    item.community_rating = entry.community_rating;
    item.production_year = entry.year.map(|year| year as i32);
    item.premiere_date = entry.year.map(|year| format!("{year}-01-01T00:00:00.000Z"));
    item.run_time_ticks = entry.runtime_minutes.map(ticks_from_minutes);
}

pub fn movie_item(entry: &CatalogEntry, server: &str) -> BaseItemDto {
    let id = ItemId::from_atlas_id(&crate::engines::identity::AtlasID::IMDb {
        id: entry.imdb_id.clone(),
        season: None,
        episode: None,
    });

    let mut item = base(
        id.to_hex(),
        entry.name.clone(),
        server.to_string(),
        &entry.imdb_id,
    );
    apply_entry(&mut item, entry);
    item.item_type = "Movie".to_string();
    item.media_type = "Video".to_string();
    item.is_folder = false;
    item.parent_id = Some(ItemId::library(Library::Movies).to_hex());
    item.primary_image_aspect_ratio = Some(0.666_666_7);
    item
}

pub fn series_item(entry: &CatalogEntry, server: &str) -> BaseItemDto {
    let id = ItemId::series(Namespace::Imdb, imdb_number(&entry.imdb_id));

    let mut item = base(
        id.to_hex(),
        entry.name.clone(),
        server.to_string(),
        &entry.imdb_id,
    );
    apply_entry(&mut item, entry);
    item.item_type = "Series".to_string();
    item.media_type = "Unknown".to_string();
    item.is_folder = true;
    item.parent_id = Some(ItemId::library(Library::Shows).to_hex());
    item.primary_image_aspect_ratio = Some(0.666_666_7);
    // A series is navigational: its runtime belongs to its episodes.
    item.run_time_ticks = None;
    item
}

/// Dispatches on the catalogue's own type, so a series never renders as a film.
pub fn catalog_item(entry: &CatalogEntry, server: &str) -> BaseItemDto {
    if entry.is_series() {
        series_item(entry, server)
    } else {
        movie_item(entry, server)
    }
}

pub fn season_name(season: u32) -> String {
    if season == 0 {
        // Cinemeta files specials under season 0, and "Season 0" reads as a bug.
        "Specials".to_string()
    } else {
        format!("Season {season}")
    }
}

pub fn season_item(series: &CatalogEntry, season: u32, server: &str) -> BaseItemDto {
    let payload = imdb_number(&series.imdb_id);
    let id = ItemId::season(Namespace::Imdb, payload, season as u16);

    let mut item = base(
        id.to_hex(),
        season_name(season),
        server.to_string(),
        &series.imdb_id,
    );
    item.item_type = "Season".to_string();
    item.media_type = "Unknown".to_string();
    item.is_folder = true;
    item.index_number = Some(season as i32);
    item.series_name = Some(series.name.clone());
    item.series_id = Some(ItemId::series(Namespace::Imdb, payload).to_hex());
    item.parent_id = item.series_id.clone();
    item.primary_image_aspect_ratio = Some(0.666_666_7);
    item
}

pub fn episode_item(series: &CatalogEntry, episode: &EpisodeMeta, server: &str) -> BaseItemDto {
    let payload = imdb_number(&series.imdb_id);
    let id = ItemId::episode(
        Namespace::Imdb,
        payload,
        episode.season as u16,
        episode.episode as u16,
    );

    let name = episode
        .name
        .clone()
        .unwrap_or_else(|| format!("Episode {}", episode.episode));

    let mut item = base(id.to_hex(), name, server.to_string(), &series.imdb_id);
    item.item_type = "Episode".to_string();
    item.media_type = "Video".to_string();
    item.is_folder = false;
    item.overview = episode.overview.clone();
    item.premiere_date = episode.released.clone();
    item.run_time_ticks = episode
        .runtime_minutes
        .or(series.runtime_minutes)
        .map(ticks_from_minutes);
    // The three fields Infuse needs to place an episode in a series.
    item.index_number = Some(episode.episode as i32);
    item.parent_index_number = Some(episode.season as i32);
    item.series_name = Some(series.name.clone());
    item.series_id = Some(ItemId::series(Namespace::Imdb, payload).to_hex());
    item.season_id = Some(ItemId::season(Namespace::Imdb, payload, episode.season as u16).to_hex());
    item.season_name = Some(season_name(episode.season));
    item.parent_id = item.season_id.clone();
    item.primary_image_aspect_ratio = Some(1.777_777_8);
    item
}

#[cfg(test)]
mod tests {
    use super::{catalog_item, episode_item, season_item, season_name, ticks_from_minutes};
    use crate::api::jellyfin::ids::{ItemId, ItemKind};
    use crate::engines::catalog::{CatalogEntry, EpisodeMeta};

    fn film() -> CatalogEntry {
        CatalogEntry {
            imdb_id: "tt0133093".to_string(),
            name: "The Matrix".to_string(),
            media_type: "movie".to_string(),
            year: Some(1999),
            description: Some("A hacker learns the truth.".to_string()),
            genres: vec!["Action".to_string()],
            community_rating: Some(8.7),
            runtime_minutes: Some(136),
        }
    }

    fn show() -> CatalogEntry {
        CatalogEntry {
            imdb_id: "tt0944947".to_string(),
            name: "Game of Thrones".to_string(),
            media_type: "series".to_string(),
            year: Some(2011),
            description: None,
            genres: vec![],
            community_rating: Some(9.2),
            runtime_minutes: Some(57),
        }
    }

    #[test]
    fn ticks_are_hundred_nanosecond_units() {
        // A wrong scale here breaks every scrubber and resume point.
        assert_eq!(ticks_from_minutes(1), 600_000_000);
        assert_eq!(ticks_from_minutes(136), 81_600_000_000);
    }

    #[test]
    fn films_and_series_get_different_kinds_from_one_catalog() {
        let movie = catalog_item(&film(), "server");
        let series = catalog_item(&show(), "server");

        assert_eq!(movie.item_type, "Movie");
        assert!(!movie.is_folder);
        assert_eq!(series.item_type, "Series");
        assert!(series.is_folder);

        assert_eq!(
            ItemId::parse(&movie.id).map(|id| id.kind),
            Some(ItemKind::Movie)
        );
        assert_eq!(
            ItemId::parse(&series.id).map(|id| id.kind),
            Some(ItemKind::Series)
        );
    }

    #[test]
    fn a_series_carries_no_runtime_of_its_own() {
        // Runtime belongs to the episodes; setting it on the series makes
        // clients show a progress bar for a folder.
        assert_eq!(catalog_item(&show(), "server").run_time_ticks, None);
        assert_eq!(
            catalog_item(&film(), "server").run_time_ticks,
            Some(81_600_000_000)
        );
    }

    #[test]
    fn episodes_carry_the_three_fields_infuse_places_them_by() {
        let episode = episode_item(
            &show(),
            &EpisodeMeta {
                season: 1,
                episode: 2,
                name: Some("The Kingsroad".to_string()),
                overview: None,
                released: None,
                runtime_minutes: Some(56),
            },
            "server",
        );

        assert_eq!(episode.index_number, Some(2));
        assert_eq!(episode.parent_index_number, Some(1));
        assert_eq!(episode.series_name.as_deref(), Some("Game of Thrones"));
        assert!(episode.series_id.is_some());
        assert!(episode.season_id.is_some());
    }

    #[test]
    fn episode_ids_agree_with_the_season_and_series_they_name() {
        let episode = episode_item(
            &show(),
            &EpisodeMeta {
                season: 1,
                episode: 2,
                name: None,
                overview: None,
                released: None,
                runtime_minutes: None,
            },
            "server",
        );

        let decoded = ItemId::parse(&episode.id).expect("an episode id must decode");
        assert_eq!(
            decoded.series_id().map(ItemId::to_hex),
            episode.series_id.clone()
        );
        assert_eq!(
            decoded.season_id().map(ItemId::to_hex),
            episode.season_id.clone()
        );
        // Parent is the season, which is how a client walks back up the tree.
        assert_eq!(episode.parent_id, episode.season_id);
    }

    #[test]
    fn an_unnamed_episode_still_gets_a_usable_title() {
        let episode = episode_item(
            &show(),
            &EpisodeMeta {
                season: 2,
                episode: 7,
                name: None,
                overview: None,
                released: None,
                runtime_minutes: None,
            },
            "server",
        );

        assert_eq!(episode.name, "Episode 7");
        // Falls back to the series runtime rather than leaving it unknown.
        assert_eq!(episode.run_time_ticks, Some(ticks_from_minutes(57)));
    }

    #[test]
    fn season_zero_is_named_specials() {
        assert_eq!(season_name(0), "Specials");
        assert_eq!(season_name(3), "Season 3");
        assert_eq!(season_item(&show(), 0, "server").index_number, Some(0));
    }

    #[test]
    fn items_advertise_artwork_so_clients_request_it() {
        let movie = catalog_item(&film(), "server");

        assert_eq!(
            movie.image_tags.get("Primary").map(String::as_str),
            Some("tt0133093")
        );
        assert!(!movie.backdrop_image_tags.is_empty());
        assert_eq!(
            movie.provider_ids.get("Imdb").map(String::as_str),
            Some("tt0133093")
        );
    }

    #[test]
    fn browsing_results_never_carry_media_sources() {
        // Filling these during enumeration would fire a provider search per tile.
        assert!(catalog_item(&film(), "server").media_sources.is_empty());
        assert!(catalog_item(&show(), "server").media_sources.is_empty());
    }
}
