//! Browsing, searching, and item detail.
//!
//! Nothing here imports `engines::playback`. A listing that hydrated sources
//! would fire a provider search for every tile on screen; sources are resolved
//! in exactly one place, and it is not this one.

use crate::api::jellyfin::auth::AuthContext;
use crate::api::jellyfin::dto::{BaseItemDto, QueryResult};
use crate::api::jellyfin::ids::{ItemId, ItemKind, Library};
use crate::api::jellyfin::map::{
    apply_user_data, catalog_item, episode_item, season_item, with_user_data,
};
use crate::api::jellyfin::query::JellyfinQuery;
use crate::api::jellyfin::shows::{episodes_for, seasons_for};
use crate::engines::catalog::{
    catalog_slice, search, series_meta, title_meta, CatalogKind, MediaKind,
};
use axum::{
    extract::{Path, Query},
    routing::get,
    Json, Router,
};
use std::collections::HashMap;

/// How far Library Mode is allowed to walk. Direct Mode fetches on demand and
/// needs no ceiling, but a full sync against a catalogue with no natural end
/// would page forever.
const LIBRARY_MODE_CAP: usize = 100;

/// Item types Atlas can serve. A request for anything else gets an honest empty
/// page rather than films relabelled as albums.
const SERVED_TYPES: [&str; 4] = ["Movie", "Series", "Season", "Episode"];

pub fn router() -> Router {
    Router::new()
        .route("/Users/:user_id/Items", get(items))
        .route("/Users/:user_id/Items/Latest", get(latest))
        .route("/Users/:user_id/Items/Resume", get(resume))
        .route("/Users/:user_id/Items/:item_id", get(item_detail))
        .route("/Items/:item_id", get(item_detail_flat))
}

/// Jellyfin has no "there may be more" flag, so the count carries that meaning:
/// a full page reports room beyond it, and a short page reports the end. Getting
/// this wrong either truncates a library or pages it forever.
fn reported_total(start: usize, returned: usize, limit: usize, exhausted: bool) -> i32 {
    if exhausted || returned < limit {
        (start + returned) as i32
    } else {
        (start + returned + limit) as i32
    }
}

fn page(
    items: Vec<BaseItemDto>,
    start: usize,
    limit: usize,
    exhausted: bool,
) -> QueryResult<BaseItemDto> {
    let total = reported_total(start, items.len(), limit, exhausted);
    QueryResult::new(items, total, start as i32)
}

/// A row that explains why a Library Mode sync sees so little.
///
/// Atlas has no library to enumerate — the catalogue is fetched on demand — so
/// a full sync walks a list with no natural end. Direct Mode is the mode this
/// server is built for, and saying so where the viewer is looking beats a
/// library that is quietly a hundred items long.
fn library_mode_notice(server: &str) -> BaseItemDto {
    let mut item = BaseItemDto::folder(
        ItemId::root().to_hex(),
        "Switch Infuse to Direct Mode".to_string(),
        server.to_string(),
    );
    item.item_type = "Folder".to_string();
    item.overview = Some(
        "Atlas fetches titles on demand and has no fixed library to sync, so \
         Library Mode only ever sees a small sample. Edit this server in \
         Infuse, open the Advanced tab, and turn Library Mode off to browse \
         everything."
            .to_string(),
    );
    item
}

/// Turns stored item ids back into full items, dropping any that no longer
/// resolve — a title can disappear from upstream between sessions.
async fn hydrate(auth: &AuthContext, ids: &[String], server: &str) -> Vec<BaseItemDto> {
    let lookups = ids.iter().map(|item_id| build_item_detail(item_id, server));

    let items: Vec<BaseItemDto> = futures::future::join_all(lookups)
        .await
        .into_iter()
        .flatten()
        .collect();

    with_user_data(items, &auth.token).await
}

/// Slices an already-materialised list, used wherever the whole set is known
/// up front (seasons, episodes) rather than paged from upstream.
fn slice(all: Vec<BaseItemDto>, start: usize, limit: usize) -> QueryResult<BaseItemDto> {
    let total = all.len() as i32;
    let items = all.into_iter().skip(start).take(limit).collect();
    QueryResult::new(items, total, start as i32)
}

async fn items(
    auth: AuthContext,
    Path(_user_id): Path<String>,
    Query(raw): Query<HashMap<String, String>>,
) -> Json<QueryResult<BaseItemDto>> {
    let query = JellyfinQuery::from_map(raw);
    let server = auth.server_id();
    let start = query.start_index();

    // Library Mode enumerates; bound it and report the count honestly so the
    // client stops rather than walking forever.
    let library_mode = auth.mode().enumerates_library();
    let limit = if library_mode {
        query.limit().min(LIBRARY_MODE_CAP)
    } else {
        query.limit()
    };

    if library_mode && start >= LIBRARY_MODE_CAP {
        return Json(QueryResult::new(
            Vec::new(),
            LIBRARY_MODE_CAP as i32,
            start as i32,
        ));
    }

    // Library Mode is bounded rather than refused — mode is partly a user
    // setting and a hard failure reads as a broken server — but a truncated
    // library with no explanation reads as one too. Leading the first page with
    // a note makes the cause visible where the viewer is already looking.
    let mut notice = Vec::new();
    if library_mode && start == 0 {
        notice.push(library_mode_notice(&server));
    }

    // Asked only for things Atlas does not carry — music, photos, live TV.
    if query.wants_none_of(&SERVED_TYPES) {
        return Json(QueryResult::empty());
    }

    // The Favorites row, which Direct Mode surfaces on the home screen.
    if query
        .list("Filters")
        .iter()
        .any(|filter| filter.eq_ignore_ascii_case("IsFavorite"))
    {
        let ids = crate::engines::playstate::favorite_items(&auth.token, limit).await;
        return Json(QueryResult::complete(hydrate(&auth, &ids, &server).await));
    }

    if let Some(term) = query.search_term() {
        let mut results = Vec::new();
        if query.includes_type("Movie") {
            for entry in search(MediaKind::Movie, term, limit).await {
                results.push(catalog_item(&entry, &server));
            }
        }
        if query.includes_type("Series") {
            for entry in search(MediaKind::Series, term, limit).await {
                results.push(catalog_item(&entry, &server));
            }
        }
        return Json(slice(results, 0, limit));
    }

    let parent = query.parent_id().and_then(ItemId::parse);

    let result = match parent {
        Some(id) => match (id.as_library(), id.kind) {
            (Some(Library::Movies), _) => {
                shelf(
                    MediaKind::Movie,
                    &query,
                    start,
                    limit,
                    &server,
                    library_mode,
                )
                .await
            }
            (Some(Library::Shows), _) => {
                shelf(
                    MediaKind::Series,
                    &query,
                    start,
                    limit,
                    &server,
                    library_mode,
                )
                .await
            }
            (None, ItemKind::Series) => match id.imdb_id() {
                Some(imdb_id) => slice(seasons_for(&imdb_id, &server).await, start, limit),
                None => QueryResult::empty(),
            },
            (None, ItemKind::Season) => match id.imdb_id() {
                Some(imdb_id) => slice(
                    episodes_for(&imdb_id, id.season.map(u32::from), &server).await,
                    start,
                    limit,
                ),
                None => QueryResult::empty(),
            },
            _ => QueryResult::empty(),
        },
        // No parent: infer the shelf from the requested type, defaulting to
        // films, which is what a client asking for "everything" expects first.
        None => {
            let media = if query.includes_type("Movie") {
                MediaKind::Movie
            } else {
                MediaKind::Series
            };
            shelf(media, &query, start, limit, &server, library_mode).await
        }
    };

    // Attached last so every path gets it, in one snapshot for the page.
    let mut items = with_user_data(result.items, &auth.token).await;
    let total = result.total_record_count + notice.len() as i32;
    notice.append(&mut items);

    Json(QueryResult::new(notice, total, result.start_index))
}

async fn shelf(
    media: MediaKind,
    query: &JellyfinQuery,
    start: usize,
    limit: usize,
    server: &str,
    library_mode: bool,
) -> QueryResult<BaseItemDto> {
    let kind = match query.get("SortBy").map(str::to_ascii_lowercase).as_deref() {
        Some(sort) if sort.contains("datecreated") || sort.contains("premieredate") => {
            CatalogKind::New
        }
        Some(sort) if sort.contains("communityrating") => CatalogKind::Featured,
        _ => CatalogKind::Popular,
    };

    let entries = catalog_slice(media, kind, start, limit).await;
    let items: Vec<_> = entries
        .iter()
        .map(|entry| catalog_item(entry, server))
        .collect();

    page(items, start, limit, library_mode)
}

/// The "recently added" row. Jellyfin expects a bare array here, not the paged
/// envelope every other listing uses.
async fn latest(
    auth: AuthContext,
    Path(_user_id): Path<String>,
    Query(raw): Query<HashMap<String, String>>,
) -> Json<Vec<BaseItemDto>> {
    let query = JellyfinQuery::from_map(raw);
    let server = auth.server_id();
    let limit = query.limit();

    let media = match query
        .parent_id()
        .and_then(ItemId::parse)
        .and_then(|id| id.as_library())
    {
        Some(Library::Shows) => MediaKind::Series,
        Some(Library::Movies) => MediaKind::Movie,
        None => {
            if query.includes_type("Series") && !query.includes_type("Movie") {
                MediaKind::Series
            } else {
                MediaKind::Movie
            }
        }
    };

    let entries = catalog_slice(media, CatalogKind::New, 0, limit).await;
    Json(
        entries
            .iter()
            .map(|entry| catalog_item(entry, &server))
            .collect(),
    )
}

/// Empty until playback state is stored.
async fn resume(
    auth: AuthContext,
    Path(_user_id): Path<String>,
    Query(raw): Query<HashMap<String, String>>,
) -> Json<QueryResult<BaseItemDto>> {
    let query = JellyfinQuery::from_map(raw);
    let ids = crate::engines::playstate::resumable(&auth.token, query.limit()).await;
    let server = auth.server_id();

    Json(QueryResult::complete(hydrate(&auth, &ids, &server).await))
}

async fn item_detail(
    auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
) -> Json<Option<BaseItemDto>> {
    Json(detail_with_prewarm(&auth, &item_id).await)
}

/// The newer flat form, delegating to the same lookup so both work.
async fn item_detail_flat(
    auth: AuthContext,
    Path(item_id): Path<String>,
) -> Json<Option<BaseItemDto>> {
    Json(detail_with_prewarm(&auth, &item_id).await)
}

/// Opening an item page usually means Play is seconds away, so source
/// resolution starts now rather than behind a spinner later.
///
/// Note the call goes through `jellyfin::playback`, not `engines::playback`:
/// this module must stay unable to resolve anything of its own accord.
async fn detail_with_prewarm(auth: &AuthContext, item_id: &str) -> Option<BaseItemDto> {
    if let Some(id) = ItemId::parse(item_id) {
        crate::api::jellyfin::playback::prewarm(auth, &id);
    }

    let mut item = build_item_detail(item_id, &auth.server_id()).await?;
    let state = crate::engines::playstate::state_for(&auth.token, item_id).await;
    apply_user_data(&mut item, &state);

    Some(item)
}

async fn build_item_detail(item_id: &str, server: &str) -> Option<BaseItemDto> {
    let id = ItemId::parse(item_id)?;

    match id.kind {
        ItemKind::Library => {
            let library = id.as_library()?;
            let mut item = BaseItemDto::folder(
                id.to_hex(),
                library.display_name().to_string(),
                server.to_string(),
            );
            item.item_type = "CollectionFolder".to_string();
            item.collection_type = Some(library.collection_type().to_string());
            Some(item)
        }
        ItemKind::Root => Some(BaseItemDto::folder(
            id.to_hex(),
            "Atlas".to_string(),
            server.to_string(),
        )),
        ItemKind::Movie => {
            let entry = title_meta(&id.imdb_id()?, MediaKind::Movie).await?;
            Some(catalog_item(&entry, server))
        }
        ItemKind::Series => {
            let entry = title_meta(&id.imdb_id()?, MediaKind::Series).await?;
            Some(catalog_item(&entry, server))
        }
        ItemKind::Season => {
            let meta = series_meta(&id.imdb_id()?).await?;
            Some(season_item(&meta.entry, u32::from(id.season?), server))
        }
        ItemKind::Episode => {
            let meta = series_meta(&id.imdb_id()?).await?;
            let season = u32::from(id.season?);
            let episode = u32::from(id.episode?);
            let video = meta
                .videos
                .iter()
                .find(|video| video.season == season && video.episode == episode)?;
            Some(episode_item(&meta.entry, video, server))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{page, reported_total, slice, LIBRARY_MODE_CAP};
    use crate::api::jellyfin::dto::BaseItemDto;

    fn items(count: usize) -> Vec<BaseItemDto> {
        (0..count)
            .map(|index| {
                BaseItemDto::folder(
                    format!("id-{index}"),
                    format!("Item {index}"),
                    "server".to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn a_full_page_reports_room_beyond_it() {
        // Jellyfin has no "more available" flag; the count carries that meaning.
        assert_eq!(reported_total(0, 50, 50, false), 100);
        assert_eq!(reported_total(50, 50, 50, false), 150);
    }

    #[test]
    fn a_short_page_reports_the_end_of_the_list() {
        assert_eq!(reported_total(0, 12, 50, false), 12);
        assert_eq!(reported_total(100, 3, 50, false), 103);
    }

    #[test]
    fn library_mode_reports_exactly_what_it_returned() {
        // This is what stops a full sync walking a catalogue with no end.
        assert_eq!(reported_total(0, 50, 50, true), 50);
        assert_eq!(reported_total(50, 50, 50, true), 100);
    }

    #[test]
    fn paging_preserves_the_clients_offset() {
        let result = page(items(50), 100, 50, false);

        assert_eq!(result.start_index, 100);
        assert_eq!(result.items.len(), 50);
    }

    #[test]
    fn slicing_a_known_list_reports_its_true_length() {
        // Seasons and episodes are fully known, so the count is exact.
        let result = slice(items(10), 3, 4);

        assert_eq!(result.total_record_count, 10);
        assert_eq!(result.start_index, 3);
        assert_eq!(result.items.len(), 4);
        assert_eq!(result.items[0].name, "Item 3");
    }

    #[test]
    fn slicing_past_the_end_yields_an_empty_page_not_an_error() {
        let result = slice(items(5), 50, 10);

        assert!(result.items.is_empty());
        assert_eq!(result.total_record_count, 5);
    }

    #[test]
    fn the_library_mode_cap_is_a_round_bound() {
        assert_eq!(LIBRARY_MODE_CAP, 100);
    }
}
