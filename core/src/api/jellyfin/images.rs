//! Artwork.
//!
//! Metahub URLs are derived from the IMDb id, so a poster needs no catalogue
//! lookup at all.
//!
//! These originally answered with a redirect, to keep image bytes off Atlas the
//! way playback keeps video off it. Infuse does not follow it: Jellyfin's own
//! image endpoint returns bytes, so a client has no reason to expect anything
//! else, and every poster came back blank. The bytes are proxied instead.
//!
//! That is affordable in a way video is not — a poster is tens of kilobytes,
//! it is fetched through the catalogue's own connection pool rather than the
//! one playback shares, and the cache headers below mean a client asks once.
//!
//! These handlers take no `AuthContext`, so core does not gate them — but the
//! gateway does, and it is the only public way in, so in practice artwork needs
//! a token like everything else. Infuse sends one. Core stays ungated so the
//! surface can be exercised directly in development, and because an item id
//! reveals nothing a catalogue listing did not already.

use crate::api::jellyfin::ids::ItemId;
use crate::engines::catalog::{backdrop_url, image_bytes, logo_url, poster_url};
use axum::{
    extract::Path,
    http::{header, StatusCode},
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

/// The upstream artwork an item maps to, if any.
///
/// Seasons and episodes have no artwork of their own in Cinemeta, so they
/// inherit the series poster rather than showing nothing.
pub fn artwork_url(item_id: &str, image_type: &str) -> Option<String> {
    let imdb_id = ItemId::parse(item_id)?.imdb_id()?;

    Some(match image_type.to_ascii_lowercase().as_str() {
        "backdrop" | "art" | "thumb" => backdrop_url(&imdb_id),
        "logo" => logo_url(&imdb_id),
        _ => poster_url(&imdb_id),
    })
}

async fn serve(item_id: &str, image_type: &str) -> Response {
    let Some(url) = artwork_url(item_id, image_type) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Some((content_type, bytes)) = image_bytes(&url).await else {
        // A missing poster is not an error worth surfacing; the client simply
        // shows its own placeholder.
        return StatusCode::NOT_FOUND.into_response();
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            // Artwork for a title does not change, so ask once.
            (
                header::CACHE_CONTROL,
                "public, max-age=604800, immutable".to_string(),
            ),
        ],
        bytes,
    )
        .into_response()
}

async fn image(Path((item_id, image_type)): Path<(String, String)>) -> Response {
    serve(&item_id, &image_type).await
}

async fn indexed_image(
    Path((item_id, image_type, _index)): Path<(String, String, String)>,
) -> Response {
    serve(&item_id, &image_type).await
}

#[cfg(test)]
mod tests {
    use super::artwork_url;
    use crate::api::jellyfin::ids::{ItemId, Library, Namespace};

    #[test]
    fn posters_and_backdrops_map_to_their_own_metahub_paths() {
        let series = ItemId::series(Namespace::Imdb, 133_093).to_hex();

        assert_eq!(
            artwork_url(&series, "Primary").as_deref(),
            Some("https://images.metahub.space/poster/medium/tt0133093/img")
        );
        assert_eq!(
            artwork_url(&series, "Backdrop").as_deref(),
            Some("https://images.metahub.space/background/medium/tt0133093/img")
        );
    }

    #[test]
    fn episodes_fall_back_to_the_series_artwork() {
        // Cinemeta publishes no per-episode stills, and a blank tile is worse.
        let episode = ItemId::episode(Namespace::Imdb, 944_947, 1, 2).to_hex();

        assert_eq!(
            artwork_url(&episode, "Primary").as_deref(),
            Some("https://images.metahub.space/poster/medium/tt0944947/img")
        );
    }

    #[test]
    fn an_unknown_image_type_still_yields_a_poster() {
        let series = ItemId::series(Namespace::Imdb, 944_947).to_hex();

        assert!(artwork_url(&series, "Banner").is_some());
    }

    #[test]
    fn ids_that_are_not_ours_have_no_artwork() {
        assert_eq!(
            artwork_url("f137a2dd21bbc1b99aa5c0f6bf02a805", "Primary"),
            None
        );
        // A library folder has no IMDb id behind it.
        assert_eq!(
            artwork_url(&ItemId::library(Library::Movies).to_hex(), "Primary"),
            None
        );
    }
}
