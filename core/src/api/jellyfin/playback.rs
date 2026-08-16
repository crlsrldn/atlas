//! Source selection and the redirect that starts playback.
//!
//! This is the only module in the Jellyfin surface allowed to resolve sources.
//! Browsing must never reach `engines::playback`, or a screen of tiles becomes
//! a screen of provider searches.
//!
//! The bytes never touch Atlas. `PlaybackInfo` hands back versions whose paths
//! point here; a client fetching one gets a 302 onward to the gateway, which
//! redirects again to the CDN, and the player range-requests that directly.

use crate::api::config::UserPreferences;
use crate::api::jellyfin::auth::AuthContext;
use crate::api::jellyfin::dto::{MediaSourceInfo, PlaybackInfoRequest, PlaybackInfoResponse};
use crate::api::jellyfin::ids::ItemId;
use crate::api::jellyfin::map::{cached_first, media_source, ticks_from_minutes};
use crate::api::jellyfin::query::JellyfinQuery;
use crate::api::jellyfin::system::public_base_url;
use crate::engines::identity::AtlasID;
use crate::engines::playback::{resolve_detailed_streams_with_preferences, DetailedStream};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Shared with the Stremio surface on purpose. Whether a torrent is alive is a
/// property of the torrent, not of the client that asked for it, so both
/// surfaces should learn from the same evidence.
const HISTORY_SCOPE: &str = "global";

pub fn router() -> Router {
    Router::new()
        .route(
            "/Items/:item_id/PlaybackInfo",
            get(playback_info).post(playback_info),
        )
        .route("/Videos/:item_id/stream", get(stream).head(stream))
        .route(
            "/Videos/:item_id/stream.:container",
            get(stream).head(stream),
        )
}

/// One lock per item, so concurrent requests for the same thing resolve it once
/// and everyone else reads the cache it leaves behind.
///
/// This is what makes prewarming safe. Measured on a cold cache: resolving
/// takes ~700ms, and a fire-and-forget prewarm made an immediate Play *slower*
/// — 2.8s — because the prewarm and the real request each resolved
/// independently and fought over the shared HTTP client. Waiting on the
/// in-flight work instead costs at worst the ~700ms it would have cost anyway,
/// and turns a viewer who paused to read the synopsis into a 34ms start.
///
/// Deliberately its own map rather than `engines::cache`, whose single mutex is
/// on the Stremio resolve path.
type InFlight = Arc<tokio::sync::Mutex<()>>;
static IN_FLIGHT: Lazy<Mutex<HashMap<String, (InFlight, Instant)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// How long an entry is kept before it is assumed finished and evicted.
const IN_FLIGHT_WINDOW: Duration = Duration::from_secs(120);

/// The lock covering resolution of one item for one caller's credentials.
fn in_flight_for(key: &str) -> InFlight {
    let Ok(mut map) = IN_FLIGHT.lock() else {
        // A poisoned map must not stop playback; resolve without coordination.
        return Arc::new(tokio::sync::Mutex::new(()));
    };

    let now = Instant::now();
    // Entries someone still holds are kept regardless of age.
    map.retain(|_, (lock, claimed)| {
        Arc::strong_count(lock) > 1 || now.duration_since(*claimed) < IN_FLIGHT_WINDOW
    });

    map.entry(key.to_string())
        .or_insert_with(|| (Arc::new(tokio::sync::Mutex::new(())), now))
        .0
        .clone()
}

fn in_flight_key(auth: &AuthContext, item: &ItemId) -> String {
    // Keyed by credentials too: preferences change which sources exist.
    format!("{}:{}", auth.token, item.to_hex())
}

/// Warms the source cache for an item a viewer is looking at.
///
/// The one sanctioned exception to "browsing never resolves". In Stremio the
/// viewer reads a list of streams while resolution happens; in Infuse,
/// `PlaybackInfo` fires *after* Play is pressed, behind a spinner. Opening an
/// item page is a strong signal that Play is seconds away, so the work starts
/// then and the result is thrown away — only the cache matters.
///
/// Never in Library Mode, where "opening an item" is a sync walking the whole
/// catalogue and this would become a provider search per title.
pub fn prewarm(auth: &AuthContext, item: &ItemId) {
    if auth.mode().enumerates_library() {
        return;
    }

    let Some(atlas_id) = item.to_playable_atlas_id() else {
        return;
    };

    let lock = in_flight_for(&in_flight_key(auth, item));
    // Already resolving; a second pass would only duplicate the work.
    let Ok(held) = lock.clone().try_lock_owned() else {
        return;
    };

    let prefs = capability_adjusted_preferences(auth, None);
    let monetization = auth.monetization_enabled;
    let token = auth.token.clone();

    tokio::spawn(async move {
        let started = Instant::now();
        let streams = resolve_detailed_streams_with_preferences(
            atlas_id,
            prefs,
            monetization,
            HISTORY_SCOPE,
            Some(&token),
        )
        .await;

        tracing::debug!(
            sources = streams.len(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            "prewarmed sources ahead of playback"
        );
        drop(held);
    });
}

/// Resolves the ranked sources for an item, cached ones first.
///
/// Waits on any prewarm already running for this item rather than starting a
/// second resolve alongside it.
async fn sources_for(
    auth: &AuthContext,
    item: &ItemId,
    atlas_id: AtlasID,
    prefs: UserPreferences,
) -> Vec<DetailedStream> {
    let lock = in_flight_for(&in_flight_key(auth, item));
    let _held = lock.lock().await;

    let streams = resolve_detailed_streams_with_preferences(
        atlas_id,
        prefs,
        auth.monetization_enabled,
        HISTORY_SCOPE,
        Some(&auth.token),
    )
    .await;

    cached_first(streams)
}

/// Applies what is known about the device to the user's preferences.
///
/// `ai_decision` matches on User-Agent fragments, and Infuse's agent
/// (`Infuse-Direct/7.7`) matches none of its rules — an Apple TV would be
/// offered AV1 it may not decode. The device name from the auth header is the
/// better signal, and a posted device profile better still.
fn capability_adjusted_preferences(
    auth: &AuthContext,
    request: Option<&PlaybackInfoRequest>,
) -> UserPreferences {
    let mut prefs = crate::engines::ai_decision::infer_capabilities(
        &auth.capability_hint(),
        auth.prefs.clone(),
    );

    let Some(request) = request else {
        return prefs;
    };

    // A profile that lists codecs is stating fact; trust it over inference in
    // both directions.
    if let Some(profile) = &request.device_profile {
        if let Some(supported) = profile.supports_video_codec("av1") {
            prefs.exclude_av1 = !supported;
        }
        if let Some(supported) = profile.supports_video_codec("hevc") {
            prefs.exclude_hevc = !supported;
        }
    }

    let ceiling = request
        .max_streaming_bitrate
        .or_else(|| request.device_profile.as_ref()?.max_streaming_bitrate);

    if let Some(bits_per_second) = ceiling.filter(|value| *value > 0) {
        tracing::debug!(
            bits_per_second,
            "client declared a streaming bitrate ceiling"
        );
    }

    prefs
}

async fn playback_info(
    auth: AuthContext,
    Path(item_id): Path<String>,
    body: Option<Json<PlaybackInfoRequest>>,
) -> Json<PlaybackInfoResponse> {
    let session = auth.session_id();

    let parsed = ItemId::parse(&item_id);
    let Some((item, atlas_id)) =
        parsed.and_then(|item| item.to_playable_atlas_id().map(|atlas_id| (item, atlas_id)))
    else {
        // Series and seasons are navigational, and this is the guard that stops
        // a client asking one of them to play.
        return Json(PlaybackInfoResponse {
            media_sources: Vec::new(),
            play_session_id: session,
            error_code: Some("NoCompatibleStream".to_string()),
        });
    };

    let request = body.map(|Json(request)| request);
    let prefs = capability_adjusted_preferences(&auth, request.as_ref());

    let streams = sources_for(&auth, &item, atlas_id.clone(), prefs).await;
    let run_time_ticks = run_time_for(&atlas_id).await;

    // Progress reports carry no runtime, so this is the moment to learn it —
    // without one there is no telling "ten minutes in" from "finished".
    crate::engines::playstate::note_runtime(&auth.token, &item_id, run_time_ticks);
    let base_url = public_base_url();

    let media_sources: Vec<MediaSourceInfo> = streams
        .iter()
        .filter(|stream| stream.hash.is_some())
        .map(|stream| media_source(stream, &item_id, run_time_ticks, &base_url))
        .collect();

    tracing::info!(
        item = %item_id,
        sources = media_sources.len(),
        cached = streams.iter().filter(|stream| stream.is_cached).count(),
        client = %auth.mode().label(),
        "Jellyfin playback info"
    );

    Json(PlaybackInfoResponse {
        // An empty list is a legitimate answer — no configured provider, or
        // nothing found — and a client shows it far better than an error.
        error_code: media_sources
            .is_empty()
            .then(|| "NoCompatibleStream".to_string()),
        media_sources,
        play_session_id: session,
    })
}

/// Item runtime, used only to fill `RunTimeTicks`. Left unset when unknown: a
/// wrong duration poisons the scrubber and every resume point.
async fn run_time_for(atlas_id: &AtlasID) -> Option<i64> {
    crate::engines::metadata::get_metadata(atlas_id)
        .await
        .runtime_minutes
        .map(ticks_from_minutes)
}

/// Hands the player onward to the source it chose.
///
/// The redirect target is the gateway URL the resolve path already builds, used
/// verbatim. Note that `SourceResult::url` points at a route that is not mounted
/// and would 404 — only `hosted_or_local_url`, which needs the install token,
/// produces a working address.
async fn stream(
    auth: AuthContext,
    Path(params): Path<HashMap<String, String>>,
    Query(raw): Query<HashMap<String, String>>,
) -> Response {
    let Some(item_id) = params.get("item_id") else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let query = JellyfinQuery::from_map(raw);

    let Some((item, atlas_id)) = ItemId::parse(item_id)
        .and_then(|item| item.to_playable_atlas_id().map(|atlas_id| (item, atlas_id)))
    else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let prefs = capability_adjusted_preferences(&auth, None);
    let streams = sources_for(&auth, &item, atlas_id, prefs).await;
    if streams.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let chosen = match query.get("MediaSourceId") {
        Some(wanted) => streams
            .iter()
            .find(|stream| stream.hash.as_deref() == Some(wanted)),
        // No explicit choice: the first entry, which cached_first guarantees is
        // playable now whenever anything is.
        None => streams.first(),
    };

    let Some(chosen) = chosen else {
        return StatusCode::NOT_FOUND.into_response();
    };

    tracing::info!(
        item = %item_id,
        cached = chosen.is_cached,
        "Jellyfin stream redirect"
    );

    (StatusCode::FOUND, [("Location", chosen.url.clone())]).into_response()
}

#[cfg(test)]
mod tests {
    use super::{capability_adjusted_preferences, HISTORY_SCOPE};
    use crate::api::config::UserPreferences;
    use crate::api::jellyfin::auth::AuthContext;
    use crate::api::jellyfin::dto::{DeviceProfile, DirectPlayProfile, PlaybackInfoRequest};

    fn auth(device: Option<&str>) -> AuthContext {
        AuthContext {
            token: "token-abc".to_string(),
            prefs: UserPreferences::default(),
            profile_name: "Atlas".to_string(),
            monetization_enabled: false,
            client: Some("Infuse-Direct".to_string()),
            device: device.map(str::to_string),
            device_id: Some("d1".to_string()),
            version: Some("8.4".to_string()),
            user_agent: Some("Infuse-Direct/7.7".to_string()),
        }
    }

    fn profile(video_codecs: &str) -> PlaybackInfoRequest {
        PlaybackInfoRequest {
            device_profile: Some(DeviceProfile {
                max_streaming_bitrate: Some(120_000_000),
                direct_play_profiles: vec![DirectPlayProfile {
                    container: Some("mkv".to_string()),
                    video_codec: Some(video_codecs.to_string()),
                    audio_codec: None,
                }],
            }),
            max_streaming_bitrate: None,
        }
    }

    #[test]
    fn an_apple_device_is_recognised_despite_infuses_user_agent() {
        // `Infuse-Direct/7.7` matches none of the ai_decision rules, so without
        // the device name an Apple TV would be offered AV1.
        let adjusted = capability_adjusted_preferences(&auth(Some("Apple TV")), None);

        assert!(adjusted.exclude_av1);
    }

    #[test]
    fn a_declared_codec_list_overrides_inference_in_both_directions() {
        let device = auth(Some("Apple TV"));

        // The client says it can decode AV1, so the Apple-device guess yields.
        let permissive = capability_adjusted_preferences(&device, Some(&profile("h264,hevc,av1")));
        assert!(!permissive.exclude_av1);

        // And a client that omits HEVC gets it excluded.
        let restrictive = capability_adjusted_preferences(&device, Some(&profile("h264")));
        assert!(restrictive.exclude_hevc);
        assert!(restrictive.exclude_av1);
    }

    #[test]
    fn an_empty_profile_claims_nothing_and_excludes_nothing() {
        let request = PlaybackInfoRequest {
            device_profile: Some(DeviceProfile::default()),
            max_streaming_bitrate: None,
        };

        let adjusted = capability_adjusted_preferences(&auth(None), Some(&request));
        assert!(!adjusted.exclude_hevc);
    }

    #[test]
    fn playback_history_is_shared_with_the_stremio_surface() {
        // A dead torrent is dead whichever client asked for it.
        assert_eq!(HISTORY_SCOPE, "global");
    }

    #[test]
    fn one_lock_covers_one_item_and_credential() {
        // Sharing the lock is what makes a real request join a prewarm rather
        // than resolve alongside it.
        let a = super::in_flight_for("token:item-a");
        let same = super::in_flight_for("token:item-a");
        let other_item = super::in_flight_for("token:item-b");
        let other_token = super::in_flight_for("other:item-a");

        assert!(std::sync::Arc::ptr_eq(&a, &same));
        assert!(!std::sync::Arc::ptr_eq(&a, &other_item));
        assert!(!std::sync::Arc::ptr_eq(&a, &other_token));
    }

    #[tokio::test]
    async fn a_second_caller_waits_for_the_first_to_finish() {
        let key = "token:item-serialised";
        let first = super::in_flight_for(key);
        let held = first.clone().lock_owned().await;

        // Anyone arriving now must wait rather than start a duplicate resolve.
        let second = super::in_flight_for(key);
        assert!(second.try_lock().is_err());

        drop(held);
        assert!(second.try_lock().is_ok());
    }

    #[tokio::test]
    async fn library_mode_never_prewarms() {
        // There, opening an item is a sync walking the catalogue, and this
        // would become one provider search per title.
        let mut library = auth(Some("Apple TV"));
        library.client = Some("Infuse-Library".to_string());

        let item = crate::api::jellyfin::ids::ItemId::from_atlas_id(
            &crate::engines::identity::AtlasID::IMDb {
                id: "tt0133093".to_string(),
                season: None,
                episode: None,
            },
        );

        super::prewarm(&library, &item);

        // Untouched, so nothing was started.
        let lock = super::in_flight_for(&super::in_flight_key(&library, &item));
        assert!(lock.try_lock().is_ok());
    }

    #[tokio::test]
    async fn navigational_items_are_never_prewarmed() {
        let series = crate::api::jellyfin::ids::ItemId::series(
            crate::api::jellyfin::ids::Namespace::Imdb,
            944_947,
        );
        let viewer = auth(Some("Apple TV"));

        super::prewarm(&viewer, &series);

        let lock = super::in_flight_for(&super::in_flight_key(&viewer, &series));
        assert!(lock.try_lock().is_ok());
    }
}
