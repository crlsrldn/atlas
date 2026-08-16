//! Seasons and episodes.
//!
//! Both come from Cinemeta's `videos[]` for a series, which is one cached fetch
//! per series and no provider traffic at all.

use crate::api::jellyfin::auth::AuthContext;
use crate::api::jellyfin::dto::{BaseItemDto, QueryResult};
use crate::api::jellyfin::ids::ItemId;
use crate::api::jellyfin::map::{episode_item, season_item};
use crate::api::jellyfin::query::JellyfinQuery;
use crate::engines::catalog::series_meta;
use axum::{
    extract::{Path, Query},
    routing::get,
    Json, Router,
};
use std::collections::HashMap;

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

/// Empty until playback state is stored. Answering with a well-formed empty
/// page is what keeps the row from erroring in the client.
async fn next_up(_auth: AuthContext) -> Json<QueryResult<BaseItemDto>> {
    Json(QueryResult::empty())
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
