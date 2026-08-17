//! Session reports, watched state, and favourites.
//!
//! One thing here must never happen: feeding [`crate::engines::history`]. A stop
//! report fires when the viewer presses stop, when the app goes to the
//! background, *and* when a stream dies — Infuse does not distinguish them — so
//! treating stops as playback failures would corrupt the source-health figures
//! that Stremio's ranking depends on. Only a genuine fetch error is evidence
//! about a source, and that is observed on the resolve path, not here.

use crate::api::jellyfin::auth::AuthContext;
use crate::api::jellyfin::dto::{JellyfinBody, UserItemDataDto};
use crate::api::jellyfin::ids::ItemId;
use crate::engines::playstate;
use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/Sessions", get(sessions))
        .route("/Sessions/Capabilities/Full", post(capabilities))
        .route("/Sessions/Playing", post(playing))
        .route("/Sessions/Playing/Progress", post(progress))
        .route("/Sessions/Playing/Stopped", post(stopped))
        .route(
            "/Users/:user_id/PlayingItems/:item_id/Progress",
            post(item_progress),
        )
        .route(
            "/Users/:user_id/PlayedItems/:item_id",
            post(mark_played).delete(clear_played),
        )
        .route(
            "/Users/:user_id/FavoriteItems/:item_id",
            post(mark_favorite).delete(clear_favorite),
        )
}

async fn sessions(_auth: AuthContext) -> Json<Vec<serde_json::Value>> {
    Json(Vec::new())
}

/// Posted immediately after authentication; a 404 puts clients in a retry loop.
async fn capabilities() -> StatusCode {
    StatusCode::NO_CONTENT
}

/// The Atlas media key an item names, for rows that are easier to read in a
/// console than a packed id.
fn atlas_key_for(item_id: &str) -> Option<String> {
    let item = ItemId::parse(item_id)?;
    item.to_playable_atlas_id()
        .map(|atlas_id| crate::engines::playback::media_key(&atlas_id))
}

fn record(auth: &AuthContext, report: &JellyfinBody) {
    let Some(item_id) = report.item_id() else {
        return;
    };
    let Some(atlas_key) = atlas_key_for(&item_id) else {
        return;
    };

    playstate::record_progress(
        &auth.token,
        &item_id,
        &atlas_key,
        report.position_ticks().unwrap_or(0),
        report.run_time_ticks(),
    );
}

async fn playing(auth: AuthContext, body: Option<Json<JellyfinBody>>) -> StatusCode {
    if let Some(Json(report)) = body {
        record(&auth, &report);
    }
    StatusCode::NO_CONTENT
}

async fn progress(auth: AuthContext, body: Option<Json<JellyfinBody>>) -> StatusCode {
    if let Some(Json(report)) = body {
        record(&auth, &report);
    }
    StatusCode::NO_CONTENT
}

/// A stop is a position report and nothing more. It carries no information
/// about whether the source was healthy — see the note at the top of this file.
async fn stopped(auth: AuthContext, body: Option<Json<JellyfinBody>>) -> StatusCode {
    if let Some(Json(report)) = body {
        record(&auth, &report);
    }
    StatusCode::NO_CONTENT
}

/// The per-item form some clients use instead of `/Sessions/Playing/Progress`.
async fn item_progress(
    auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
    body: Option<Json<JellyfinBody>>,
) -> StatusCode {
    // The item is named by the path here rather than in the body.
    let mut fields = body
        .map(|Json(report)| report.into_value())
        .unwrap_or_else(|| serde_json::json!({}));
    if let Some(object) = fields.as_object_mut() {
        object.insert("ItemId".to_string(), serde_json::Value::String(item_id));
    }
    record(&auth, &JellyfinBody::new(fields));
    StatusCode::NO_CONTENT
}

async fn state_response(auth: &AuthContext, item_id: &str) -> Json<UserItemDataDto> {
    let state = playstate::state_for(&auth.token, item_id).await;

    Json(UserItemDataDto {
        playback_position_ticks: state.position_ticks,
        play_count: state.play_count,
        is_favorite: state.is_favorite,
        played: state.played,
        played_percentage: state.played_percentage(),
        key: item_id.to_string(),
    })
}

async fn set_played(auth: &AuthContext, item_id: &str, played: bool) -> Json<UserItemDataDto> {
    if let Some(atlas_key) = atlas_key_for(item_id) {
        playstate::set_played(&auth.token, item_id, &atlas_key, played);
    }
    state_response(auth, item_id).await
}

async fn set_favorite(auth: &AuthContext, item_id: &str, favorite: bool) -> Json<UserItemDataDto> {
    if let Some(atlas_key) = atlas_key_for(item_id) {
        playstate::set_favorite(&auth.token, item_id, &atlas_key, favorite);
    }
    state_response(auth, item_id).await
}

async fn mark_played(
    auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
) -> Json<UserItemDataDto> {
    set_played(&auth, &item_id, true).await
}

async fn clear_played(
    auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
) -> Json<UserItemDataDto> {
    set_played(&auth, &item_id, false).await
}

async fn mark_favorite(
    auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
) -> Json<UserItemDataDto> {
    set_favorite(&auth, &item_id, true).await
}

async fn clear_favorite(
    auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
) -> Json<UserItemDataDto> {
    set_favorite(&auth, &item_id, false).await
}

#[cfg(test)]
mod tests {
    use super::{atlas_key_for, capabilities};
    use crate::api::jellyfin::dto::JellyfinBody;
    use crate::api::jellyfin::ids::{ItemId, Library, Namespace};
    use axum::http::StatusCode;

    #[tokio::test]
    async fn capabilities_are_accepted_rather_than_missing() {
        assert_eq!(capabilities().await, StatusCode::NO_CONTENT);
    }

    #[test]
    fn reports_are_read_whatever_casing_a_client_uses() {
        let pascal = JellyfinBody::new(serde_json::json!({"ItemId":"abc","PositionTicks":1234}));
        let camel = JellyfinBody::new(serde_json::json!({"itemId":"abc","positionTicks":1234}));

        assert_eq!(pascal.item_id().as_deref(), Some("abc"));
        assert_eq!(camel.position_ticks(), Some(1234));
    }

    #[test]
    fn a_report_missing_everything_is_still_accepted() {
        // Clients send sparse reports; refusing them produces error toasts for
        // something that is not an error.
        let empty = JellyfinBody::new(serde_json::json!({}));

        assert!(empty.item_id().is_none());
        assert_eq!(empty.position_ticks(), None);
    }

    #[test]
    fn playable_items_resolve_to_an_atlas_key() {
        let episode = ItemId::episode(Namespace::Imdb, 944_947, 1, 2).to_hex();
        let movie = ItemId::from_atlas_id(&crate::engines::identity::AtlasID::IMDb {
            id: "tt0133093".to_string(),
            season: None,
            episode: None,
        })
        .to_hex();

        assert_eq!(atlas_key_for(&episode).as_deref(), Some("tt0944947:1:2"));
        assert_eq!(atlas_key_for(&movie).as_deref(), Some("tt0133093"));
    }

    #[test]
    fn navigational_items_have_no_progress_to_record() {
        assert_eq!(
            atlas_key_for(&ItemId::series(Namespace::Imdb, 944_947).to_hex()),
            None
        );
        assert_eq!(
            atlas_key_for(&ItemId::library(Library::Movies).to_hex()),
            None
        );
        assert_eq!(atlas_key_for("not-an-atlas-id"), None);
    }
}
