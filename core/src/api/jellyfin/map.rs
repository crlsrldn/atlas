//! Turning catalogue entries into Jellyfin items.

use crate::api::jellyfin::dto::{BaseItemDto, MediaSourceInfo, MediaStream, UserItemDataDto};
use crate::api::jellyfin::ids::{ItemId, Library, Namespace};
use crate::engines::catalog::{CatalogEntry, EpisodeMeta};
use crate::engines::playback::DetailedStream;
use crate::engines::playstate::PlaybackState;

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

/// Attaches what the viewer has already done with an item.
///
/// `UserData` is what drives the resume prompt, the watched tick, and the
/// favourite heart; an item without it always looks untouched.
pub fn apply_user_data(item: &mut BaseItemDto, state: &PlaybackState) {
    item.user_data = Some(UserItemDataDto {
        playback_position_ticks: state.position_ticks,
        play_count: state.play_count,
        is_favorite: state.is_favorite,
        played: state.played,
        played_percentage: state.played_percentage(),
        key: item.id.clone(),
    });
}

/// Attaches state to a whole page at once — one snapshot for the profile
/// rather than a lookup per row.
pub async fn with_user_data(mut items: Vec<BaseItemDto>, profile_id: &str) -> Vec<BaseItemDto> {
    let ids: Vec<String> = items.iter().map(|item| item.id.clone()).collect();
    let states = crate::engines::playstate::states_for(profile_id, &ids).await;

    for item in items.iter_mut() {
        if let Some(state) = states.get(&item.id) {
            apply_user_data(item, state);
        }
    }

    items
}

/// Cached sources first, ranking order preserved within each group.
///
/// Ranking already rewards a cached source but does not guarantee it sorts
/// first, and clients auto-select index 0. That entry has to be one that plays
/// immediately whenever any such source exists.
pub fn cached_first(streams: Vec<DetailedStream>) -> Vec<DetailedStream> {
    let (cached, uncached): (Vec<_>, Vec<_>) =
        streams.into_iter().partition(|stream| stream.is_cached);
    cached.into_iter().chain(uncached).collect()
}

fn dimensions(resolution: &str) -> Option<(i32, i32)> {
    match resolution {
        "4K" => Some((3840, 2160)),
        "1080p" => Some((1920, 1080)),
        "720p" => Some((1280, 720)),
        _ => None,
    }
}

/// Atlas only ever infers these three, and clients expect ffmpeg spellings.
fn video_codec_name(codec: &str) -> Option<String> {
    match codec.to_ascii_uppercase().as_str() {
        "HEVC" => Some("hevc".to_string()),
        "AV1" => Some("av1".to_string()),
        "H264" => Some("h264".to_string()),
        _ => None,
    }
}

fn audio_codec_name(codec: &str) -> (Option<String>, Option<String>) {
    match codec.to_ascii_uppercase().as_str() {
        "TRUEHD" => (Some("truehd".to_string()), None),
        "DTS" => (Some("dts".to_string()), None),
        "AAC" => (Some("aac".to_string()), None),
        "DOLBY DIGITAL" | "AC3" => (Some("ac3".to_string()), None),
        // Atmos is a metadata layer carried on EAC3 or TrueHD, not a codec of
        // its own, so it is named in the title instead.
        "ATMOS" => (Some("eac3".to_string()), Some("Atmos".to_string())),
        "EAC3" => (Some("eac3".to_string()), None),
        _ => (None, None),
    }
}

fn channel_count(layout: &str) -> Option<i32> {
    match layout.trim() {
        "7.1" => Some(8),
        "6.1" => Some(7),
        "5.1" => Some(6),
        "2.0" | "stereo" => Some(2),
        "1.0" | "mono" => Some(1),
        _ => None,
    }
}

fn gigabytes(size_bytes: u64) -> String {
    // Decimal, matching how the figure was parsed out of the release name.
    format!("{:.1} GB", size_bytes as f64 / 1_000_000_000.0)
}

/// The line a viewer actually chooses between.
pub fn version_label(stream: &DetailedStream) -> String {
    let mut parts = vec![stream.resolution.clone(), stream.video_codec.clone()];

    if stream.has_dolby_vision {
        parts.push("DV".to_string());
    } else if stream.has_hdr {
        parts.push("HDR".to_string());
    }

    if let Some(codec) = &stream.audio_codec {
        let channels = stream
            .audio_channels
            .as_deref()
            .map(|layout| format!(" {layout}"))
            .unwrap_or_default();
        parts.push(format!("{codec}{channels}"));
    }

    if let Some(size) = stream.size_bytes.filter(|size| *size > 0) {
        parts.push(gigabytes(size));
    }

    if let Some(group) = &stream.release_group {
        parts.push(group.clone());
    }

    let prefix = if stream.is_cached {
        "⚡ "
    } else {
        // Selecting this queues the torrent instead of playing it.
        "⬇ Not cached — "
    };

    format!("{prefix}{}", parts.join(" · "))
}

fn media_streams(stream: &DetailedStream) -> Vec<MediaStream> {
    let mut video = MediaStream::video(0);
    video.codec = video_codec_name(&stream.video_codec);
    if let Some((width, height)) = dimensions(&stream.resolution) {
        video.width = Some(width);
        video.height = Some(height);
    }
    video.bit_rate = stream
        .bitrate_mbps
        .map(|mbps| (f64::from(mbps) * 1_000_000.0) as i64);
    video.video_range = Some(if stream.has_hdr { "HDR" } else { "SDR" }.to_string());
    video.video_range_type = Some(
        if stream.has_dolby_vision {
            "DOVI"
        } else if stream.has_hdr {
            "HDR10"
        } else {
            "SDR"
        }
        .to_string(),
    );
    video.display_title = Some(format!(
        "{} {}",
        stream.resolution,
        stream.video_codec.to_uppercase()
    ));

    // Always emitted, even when nothing is known: omitting it reads as a file
    // with no audio at all.
    let mut audio = MediaStream::audio(1);
    if let Some(codec) = &stream.audio_codec {
        let (name, title) = audio_codec_name(codec);
        audio.codec = name;
        audio.title = title;
    }
    if let Some(layout) = &stream.audio_channels {
        audio.channels = channel_count(layout);
        audio.channel_layout = Some(layout.clone());
    }
    audio.display_title = stream.audio_codec.clone();

    // No subtitle stream. `has_subtitles` matches bare substrings like "sub"
    // and "cc" against the title, so it fires on titles such as Succession;
    // advertising a track that is not in the container shows a broken entry.
    vec![video, audio]
}

/// Builds the entry a client sees in its version picker.
///
/// `Path` points back at Atlas rather than at the gateway or the CDN: those
/// URLs carry the install token, and the client must not hold one.
pub fn media_source(
    stream: &DetailedStream,
    item_id: &str,
    run_time_ticks: Option<i64>,
    base_url: &str,
) -> MediaSourceInfo {
    let hash = stream.hash.clone().unwrap_or_default();

    let mut source = MediaSourceInfo::direct_play(hash.clone(), version_label(stream));
    source.path = Some(format!(
        "{}/Videos/{item_id}/stream?MediaSourceId={hash}&Static=true",
        base_url.trim_end_matches('/')
    ));
    source.container = Some(
        stream
            .container
            .clone()
            // Most releases are mkv, and claiming mp4 while serving mkv makes
            // some players fail on the first range response. The reverse is
            // harmless because the container is probed on open.
            .unwrap_or_else(|| "mkv".to_string()),
    );
    source.size = stream
        .size_bytes
        .filter(|size| *size > 0)
        .map(|size| size as i64);
    source.bitrate = stream
        .bitrate_mbps
        .map(|mbps| (f64::from(mbps) * 1_000_000.0) as i64);
    source.run_time_ticks = run_time_ticks;
    source.media_streams = media_streams(stream);
    source
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

    // -----------------------------------------------------------------------
    // Playback sources
    // -----------------------------------------------------------------------

    use super::{cached_first, media_source, version_label};
    use crate::engines::playback::DetailedStream;

    fn stream(hash: &str, cached: bool) -> DetailedStream {
        DetailedStream {
            title: "The Matrix (4K)".to_string(),
            raw_title: "The.Matrix.1999.2160p.mkv".to_string(),
            container: Some("mkv".to_string()),
            provider_name: "TorBox".to_string(),
            url: "https://gateway.invalid/stremio/tok/resolve/torbox/abc/play.mp4".to_string(),
            hash: Some(hash.to_string()),
            score: 100,
            confidence: 80,
            reasons: vec![],
            resolution: "4K".to_string(),
            video_codec: "HEVC".to_string(),
            audio_codec: Some("TrueHD".to_string()),
            audio_channels: Some("7.1".to_string()),
            bitrate_mbps: Some(32.4),
            has_hdr: true,
            has_dolby_vision: true,
            has_subtitles: true,
            provider_latency_ms: Some(120),
            playback_successes: 2,
            playback_failures: 0,
            is_cached: cached,
            release_group: Some("TERMINAL".to_string()),
            size_bytes: Some(24_300_000_000),
        }
    }

    #[test]
    fn cached_sources_sort_ahead_without_disturbing_ranking_order() {
        // Clients auto-select index 0, so it must play immediately.
        let ordered = cached_first(vec![
            stream("uncached-1", false),
            stream("cached-1", true),
            stream("uncached-2", false),
            stream("cached-2", true),
        ]);

        let hashes: Vec<_> = ordered.iter().filter_map(|s| s.hash.clone()).collect();
        assert_eq!(hashes, ["cached-1", "cached-2", "uncached-1", "uncached-2"]);
    }

    #[test]
    fn the_version_label_leads_with_whether_it_will_play_now() {
        let cached = version_label(&stream("a", true));
        let uncached = version_label(&stream("a", false));

        assert!(cached.starts_with('⚡'), "got {cached}");
        assert!(uncached.contains("Not cached"), "got {uncached}");
        assert!(cached.contains("4K"));
        assert!(cached.contains("TrueHD 7.1"));
        assert!(cached.contains("24.3 GB"));
        assert!(cached.contains("TERMINAL"));
    }

    #[test]
    fn dolby_vision_wins_over_the_plain_hdr_marker() {
        let mut sdr = stream("a", true);
        sdr.has_hdr = false;
        sdr.has_dolby_vision = false;
        let mut hdr = stream("a", true);
        hdr.has_dolby_vision = false;

        assert!(version_label(&stream("a", true)).contains("DV"));
        assert!(version_label(&hdr).contains("HDR"));
        assert!(!version_label(&sdr).contains("HDR"));
    }

    #[test]
    fn a_source_points_back_at_atlas_never_at_a_url_holding_a_token() {
        let source = media_source(
            &stream("abc123", true),
            "item-1",
            Some(1_000),
            "https://atlas.invalid",
        );

        let path = source.path.expect("a source must be playable");
        assert_eq!(
            path,
            "https://atlas.invalid/Videos/item-1/stream?MediaSourceId=abc123&Static=true"
        );
        // The gateway URL carries the install token and must not leak.
        assert!(!path.contains("stremio"));
        assert!(!path.contains("play.mp4"));
    }

    #[test]
    fn sources_declare_direct_play_and_refuse_transcoding() {
        // Claiming transcoding makes a client that cannot direct-play ask for a
        // transcode URL Atlas has no way to produce.
        let source = media_source(&stream("abc", true), "item", None, "https://atlas.invalid");

        assert!(source.supports_direct_play);
        assert!(source.supports_direct_stream);
        assert!(!source.supports_transcoding);
        assert!(source.transcoding_url.is_none());
        assert!(source.is_remote);
    }

    #[test]
    fn a_missing_container_falls_back_to_mkv_not_mp4() {
        let mut unknown = stream("abc", true);
        unknown.container = None;

        let source = media_source(&unknown, "item", None, "https://atlas.invalid");
        assert_eq!(source.container.as_deref(), Some("mkv"));
    }

    #[test]
    fn synthesised_tracks_describe_the_release() {
        let source = media_source(&stream("abc", true), "item", None, "https://atlas.invalid");
        let video = &source.media_streams[0];
        let audio = &source.media_streams[1];

        assert_eq!(video.codec.as_deref(), Some("hevc"));
        assert_eq!((video.width, video.height), (Some(3840), Some(2160)));
        assert_eq!(video.video_range_type.as_deref(), Some("DOVI"));
        // A guessed profile can make a client reject a playable source.
        assert!(video.profile.is_none());

        assert_eq!(audio.codec.as_deref(), Some("truehd"));
        assert_eq!(audio.channels, Some(8));
    }

    #[test]
    fn audio_is_always_present_even_when_nothing_is_known() {
        // No audio track at all reads as a silent file.
        let mut bare = stream("abc", true);
        bare.audio_codec = None;
        bare.audio_channels = None;

        let source = media_source(&bare, "item", None, "https://atlas.invalid");
        assert_eq!(source.media_streams.len(), 2);
        assert_eq!(source.media_streams[1].stream_type, "Audio");
        assert!(source.media_streams[1].codec.is_none());
    }

    #[test]
    fn no_subtitle_track_is_advertised() {
        // has_subtitles matches substrings like "sub" and "cc" in a title, so it
        // fires on shows such as Succession. Advertising a track that is not in
        // the container shows a broken entry the viewer cannot select.
        let source = media_source(&stream("abc", true), "item", None, "https://atlas.invalid");

        assert!(!source
            .media_streams
            .iter()
            .any(|track| track.stream_type == "Subtitle"));
        assert!(source.default_subtitle_stream_index.is_none());
    }

    #[test]
    fn a_zero_size_is_reported_as_unknown_rather_than_as_zero() {
        // Sizes are scraped from release names and Some(0) is common.
        let mut sizeless = stream("abc", true);
        sizeless.size_bytes = Some(0);

        let source = media_source(&sizeless, "item", None, "https://atlas.invalid");
        assert_eq!(source.size, None);
        assert!(!version_label(&sizeless).contains("0.0 GB"));
    }

    #[test]
    fn atmos_is_carried_as_eac3_with_a_title() {
        let mut atmos = stream("abc", true);
        atmos.audio_codec = Some("Atmos".to_string());

        let source = media_source(&atmos, "item", None, "https://atlas.invalid");
        assert_eq!(source.media_streams[1].codec.as_deref(), Some("eac3"));
        assert_eq!(source.media_streams[1].title.as_deref(), Some("Atmos"));
    }
}
