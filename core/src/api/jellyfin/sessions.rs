//! Session and playback-state endpoints.
//!
//! Phase 1 accepts and discards. Infuse posts progress every few seconds and
//! marks items watched or favourite, and a 404 on any of these produces error
//! toasts and log spam even though nothing is broken. Persisting the reports is
//! Phase 4.
//!
//! One thing these must never do is feed `engines::history`. A stop report fires
//! when the viewer presses stop, when the app backgrounds, *and* when a stream
//! dies — Infuse does not distinguish them — so treating stops as playback
//! failures would corrupt the source-health statistics that Stremio's ranking
//! depends on.

use crate::api::jellyfin::auth::AuthContext;
use crate::api::jellyfin::dto::UserItemDataDto;
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

async fn playing(auth: AuthContext, body: Option<Json<serde_json::Value>>) -> StatusCode {
    log_session_event("started", &auth, body);
    StatusCode::NO_CONTENT
}

async fn progress(auth: AuthContext, body: Option<Json<serde_json::Value>>) -> StatusCode {
    log_session_event("progress", &auth, body);
    StatusCode::NO_CONTENT
}

async fn stopped(auth: AuthContext, body: Option<Json<serde_json::Value>>) -> StatusCode {
    log_session_event("stopped", &auth, body);
    StatusCode::NO_CONTENT
}

fn log_session_event(kind: &str, auth: &AuthContext, body: Option<Json<serde_json::Value>>) {
    let position = body
        .as_ref()
        .and_then(|Json(value)| value.get("PositionTicks"))
        .and_then(serde_json::Value::as_i64);

    tracing::debug!(
        event = kind,
        client = %auth.mode().label(),
        position_ticks = position,
        "Jellyfin playback report (not yet persisted)"
    );
}

fn echo_state(played: bool, is_favorite: bool, item_id: &str) -> Json<UserItemDataDto> {
    Json(UserItemDataDto {
        played,
        is_favorite,
        key: item_id.to_string(),
        ..UserItemDataDto::default()
    })
}

async fn mark_played(
    _auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
) -> Json<UserItemDataDto> {
    echo_state(true, false, &item_id)
}

async fn clear_played(
    _auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
) -> Json<UserItemDataDto> {
    echo_state(false, false, &item_id)
}

async fn mark_favorite(
    _auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
) -> Json<UserItemDataDto> {
    echo_state(false, true, &item_id)
}

async fn clear_favorite(
    _auth: AuthContext,
    Path((_user_id, item_id)): Path<(String, String)>,
) -> Json<UserItemDataDto> {
    echo_state(false, false, &item_id)
}

#[cfg(test)]
mod tests {
    use super::{capabilities, echo_state};
    use axum::http::StatusCode;

    #[tokio::test]
    async fn capabilities_are_accepted_rather_than_missing() {
        assert_eq!(capabilities().await, StatusCode::NO_CONTENT);
    }

    #[test]
    fn state_changes_echo_the_item_back_to_the_client() {
        let marked = echo_state(true, false, "abc").0;

        assert!(marked.played);
        assert!(!marked.is_favorite);
        assert_eq!(marked.key, "abc");
        assert_eq!(marked.playback_position_ticks, 0);
    }
}
