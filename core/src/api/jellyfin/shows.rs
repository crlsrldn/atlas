//! Seasons and episodes.
//!
//! Both come from Cinemeta's `videos[]` for a series, which is one cached fetch
//! per series and no provider traffic at all.

use crate::api::jellyfin::auth::AuthContext;
use crate::api::jellyfin::dto::{BaseItemDto, QueryResult};
use crate::api::jellyfin::ids::{ItemId, ItemKind};
use crate::api::jellyfin::map::{episode_item, season_item, with_user_data};
use crate::api::jellyfin::query::JellyfinQuery;
use crate::engines::catalog::series_meta;
use crate::engines::playstate;
use axum::{
    extract::{Path, Query},
    routing::get,
    Json, Router,
};
use std::collections::{HashMap, HashSet};

pub fn router() -> Router {
    Router::new()
        .route("/Shows/:series_id/Seasons", get(seasons))
        .route("/Shows/:series_id/Episodes", get(episodes))
        .route("/Shows/NextUp", get(next_up))
}

/// Resolves whatever a client put in a series path position to an IMDb id.
/// Clients pass season and episode ids here as well as series ids.
fn imdb_for(raw: &str) -> Option<String> {
    ItemId::parse(raw)?.imdb_id()
}

pub async fn seasons_for(imdb_id: &str, server: &str) -> Vec<BaseItemDto> {
    let Some(meta) = series_meta(imdb_id).await else {
        return Vec::new();
    };

    let mut seasons: Vec<u32> = meta.videos.iter().map(|video| video.season).collect();
    seasons.sort_unstable();
    seasons.dedup();

    seasons
        .into_iter()
        .map(|season| season_item(&meta.entry, season, server))
        .collect()
}

pub async fn episodes_for(imdb_id: &str, season: Option<u32>, server: &str) -> Vec<BaseItemDto> {
    let Some(meta) = series_meta(imdb_id).await else {
        return Vec::new();
    };

    let mut videos: Vec<_> = meta
        .videos
        .iter()
        .filter(|video| season.is_none_or(|wanted| video.season == wanted))
        .collect();
    videos.sort_by_key(|video| (video.season, video.episode));

    videos
        .into_iter()
        .map(|video| episode_item(&meta.entry, video, server))
        .collect()
}

async fn seasons(
    auth: AuthContext,
    Path(series_id): Path<String>,
) -> Json<QueryResult<BaseItemDto>> {
    let Some(imdb_id) = imdb_for(&series_id) else {
        return Json(QueryResult::empty());
    };

    Json(QueryResult::complete(
        seasons_for(&imdb_id, &auth.server_id()).await,
    ))
}

async fn episodes(
    auth: AuthContext,
    Path(series_id): Path<String>,
    Query(raw): Query<HashMap<String, String>>,
) -> Json<QueryResult<BaseItemDto>> {
    let Some(imdb_id) = imdb_for(&series_id) else {
        return Json(QueryResult::empty());
    };
    let query = JellyfinQuery::from_map(raw);

    // A client may name the season by id or by number.
    let season = query
        .get("SeasonId")
        .and_then(ItemId::parse)
        .and_then(|id| id.season)
        .map(u32::from)
        .or_else(|| query.number("Season").map(|value| value as u32));

    Json(QueryResult::complete(
        episodes_for(&imdb_id, season, &auth.server_id()).await,
    ))
}

/// The episode to watch next in each series the viewer has started.
///
/// Derived rather than stored: for every episode with progress, the next one in
/// Cinemeta's order is the candidate, and a series contributes at most one row.
/// A part-watched episode is itself the answer — you finish it before moving on.
async fn next_up(
    auth: AuthContext,
    Query(raw): Query<HashMap<String, String>>,
) -> Json<QueryResult<BaseItemDto>> {
    let query = JellyfinQuery::from_map(raw);
    let server = auth.server_id();

    // Recent first, so the newest activity in a series wins.
    let recent = playstate::recent_items(&auth.token, 100).await;
    let mut seen_series = HashSet::new();
    let mut next = Vec::new();

    for (item_id, state) in recent {
        if next.len() >= query.limit() {
            break;
        }

        let Some(episode) = ItemId::parse(&item_id).filter(|id| id.kind == ItemKind::Episode)
        else {
            continue;
        };
        let Some(series_id) = episode.series_id().map(|id| id.to_hex()) else {
            continue;
        };
        if !seen_series.insert(series_id) {
            continue;
        }

        // Still mid-episode: that episode is what is up next.
        let candidate = if state.is_resumable() {
            Some(episode)
        } else if state.played {
            following_episode(&episode).await
        } else {
            None
        };

        let Some(candidate) = candidate else {
            continue;
        };
        let Some(imdb_id) = candidate.imdb_id() else {
            continue;
        };

        if let Some(meta) = series_meta(&imdb_id).await {
            if let Some(video) = meta.videos.iter().find(|video| {
                Some(video.season) == candidate.season.map(u32::from)
                    && Some(video.episode) == candidate.episode.map(u32::from)
            }) {
                next.push(episode_item(&meta.entry, video, &server));
            }
        }
    }

    Json(QueryResult::complete(
        with_user_data(next, &auth.token).await,
    ))
}

/// The episode after this one in Cinemeta's ordering, crossing into the next
/// season when a finale runs out.
async fn following_episode(episode: &ItemId) -> Option<ItemId> {
    let imdb_id = episode.imdb_id()?;
    let meta = series_meta(&imdb_id).await?;

    let mut ordered: Vec<_> = meta.videos.iter().collect();
    ordered.sort_by_key(|video| (video.season, video.episode));

    let season = u32::from(episode.season?);
    let number = u32::from(episode.episode?);

    let position = ordered
        .iter()
        .position(|video| video.season == season && video.episode == number)?;
    let following = ordered.get(position + 1)?;

    Some(ItemId::episode(
        episode.namespace,
        episode.payload,
        following.season as u16,
        following.episode as u16,
    ))
}

#[cfg(test)]
mod tests {
    use super::imdb_for;
    use crate::api::jellyfin::ids::{ItemId, Namespace};

    #[test]
    fn resolves_a_series_id_to_its_imdb_id() {
        let series = ItemId::series(Namespace::Imdb, 944_947).to_hex();

        assert_eq!(imdb_for(&series), Some("tt0944947".to_string()));
    }

    #[test]
    fn resolves_season_and_episode_ids_too() {
        // Clients put more than series ids in this path position.
        let season = ItemId::season(Namespace::Imdb, 944_947, 1).to_hex();
        let episode = ItemId::episode(Namespace::Imdb, 944_947, 1, 2).to_hex();

        assert_eq!(imdb_for(&season), Some("tt0944947".to_string()));
        assert_eq!(imdb_for(&episode), Some("tt0944947".to_string()));
    }

    #[test]
    fn refuses_ids_that_are_not_ours() {
        assert_eq!(imdb_for("f137a2dd21bbc1b99aa5c0f6bf02a805"), None);
        assert_eq!(imdb_for("garbage"), None);
    }
}
