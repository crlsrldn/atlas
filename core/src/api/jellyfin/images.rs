//! Artwork.
//!
//! Metahub URLs are derived from the IMDb id, so a poster needs no catalogue
//! lookup — and redirecting rather than proxying keeps image bytes off Atlas
//! entirely, the same trick playback uses for video.
//!
//! These routes are deliberately unauthenticated. Clients fetch artwork from
//! image views that do not always carry the auth header, the destination is a
//! public CDN, and an item id reveals nothing a catalogue listing did not
//! already. Gating them would show a library of grey rectangles.

use crate::api::jellyfin::ids::ItemId;
use crate::engines::catalog::{backdrop_url, logo_url, poster_url};
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/Items/:item_id/Images/:image_type", get(image))
        .route(
            "/Items/:item_id/Images/:image_type/:index",
            get(indexed_image),
        )
}

fn redirect_for(item_id: &str, image_type: &str) -> Response {
    let Some(imdb_id) = ItemId::parse(item_id).and_then(|id| id.imdb_id()) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    // Seasons and episodes have no artwork of their own in Cinemeta, so they
    // inherit the series poster rather than showing nothing.
    let url = match image_type.to_ascii_lowercase().as_str() {
        "backdrop" | "art" | "thumb" => backdrop_url(&imdb_id),
        "logo" => logo_url(&imdb_id),
        _ => poster_url(&imdb_id),
    };

    (
        StatusCode::FOUND,
        [
            ("Location", url.as_str()),
            // Artwork for a given title never changes.
            ("Cache-Control", "public, max-age=604800"),
        ],
    )
        .into_response()
}

async fn image(Path((item_id, image_type)): Path<(String, String)>) -> Response {
    redirect_for(&item_id, &image_type)
}

async fn indexed_image(
    Path((item_id, image_type, _index)): Path<(String, String, String)>,
) -> Response {
    redirect_for(&item_id, &image_type)
}

#[cfg(test)]
mod tests {
    use super::redirect_for;
    use crate::api::jellyfin::ids::{ItemId, Namespace};
    use axum::http::StatusCode;

    fn location(item_id: &str, image_type: &str) -> Option<String> {
        let response = redirect_for(item_id, image_type);
        if response.status() != StatusCode::FOUND {
            return None;
        }
        response
            .headers()
            .get("Location")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
    }

    #[test]
    fn posters_and_backdrops_map_to_their_own_metahub_paths() {
        let movie = ItemId::series(Namespace::Imdb, 133_093).to_hex();

        assert_eq!(
            location(&movie, "Primary").as_deref(),
            Some("https://images.metahub.space/poster/medium/tt0133093/img")
        );
        assert_eq!(
            location(&movie, "Backdrop").as_deref(),
            Some("https://images.metahub.space/background/medium/tt0133093/img")
        );
    }

    #[test]
    fn episodes_fall_back_to_the_series_artwork() {
        // Cinemeta publishes no per-episode stills, and a blank tile is worse.
        let episode = ItemId::episode(Namespace::Imdb, 944_947, 1, 2).to_hex();

        assert_eq!(
            location(&episode, "Primary").as_deref(),
            Some("https://images.metahub.space/poster/medium/tt0944947/img")
        );
    }

    #[test]
    fn an_unknown_image_type_still_yields_a_poster() {
        let series = ItemId::series(Namespace::Imdb, 944_947).to_hex();

        assert!(location(&series, "Banner").is_some());
    }

    #[test]
    fn ids_that_are_not_ours_are_not_redirected() {
        assert_eq!(
            redirect_for("f137a2dd21bbc1b99aa5c0f6bf02a805", "Primary").status(),
            StatusCode::NOT_FOUND
        );
        // A library folder has no IMDb id behind it.
        let library = ItemId::library(crate::api::jellyfin::ids::Library::Movies).to_hex();
        assert_eq!(
            redirect_for(&library, "Primary").status(),
            StatusCode::NOT_FOUND
        );
    }
}
