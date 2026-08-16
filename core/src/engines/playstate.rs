//! Where a viewer got to, and what they marked as a favourite.
//!
//! Jellyfin clients expect the server to own this. Atlas had nowhere to put it:
//! [`crate::engines::history`] records whether a *source* played, not where a
//! *person* got to, and it writes to the process working directory, which on Fly
//! is per-machine and gone on redeploy.
//!
//! State is keyed on the install token — the profile — so it follows a profile
//! the way the Stremio addon URL already does.
//!
//! Two things shape the implementation. Clients report progress every few
//! seconds, so writes are debounced and always happen off the request path; and
//! every listing wants this data for every row, so reads come from an
//! in-process snapshot of the whole profile rather than a query per item.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// How long a profile snapshot is served before it is re-read.
const SNAPSHOT_TTL: Duration = Duration::from_secs(30);

/// Progress reports arrive every few seconds; persisting each one would be
/// thousands of writes per film for information nobody reads at that
/// resolution. The in-process copy still updates immediately.
const WRITE_DEBOUNCE: Duration = Duration::from_secs(10);

/// Jellyfin's convention: past this, an item counts as watched rather than
/// part-watched, and stops offering to resume a few seconds from the end.
const PLAYED_FRACTION: f64 = 0.9;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlaybackState {
    pub position_ticks: i64,
    pub runtime_ticks: Option<i64>,
    pub played: bool,
    pub play_count: i32,
    pub is_favorite: bool,
}

impl PlaybackState {
    pub fn played_percentage(&self) -> Option<f64> {
        let runtime = self.runtime_ticks.filter(|ticks| *ticks > 0)?;
        Some((self.position_ticks as f64 / runtime as f64 * 100.0).clamp(0.0, 100.0))
    }

    /// Whether this is worth offering to resume. A few seconds in is an
    /// accident, and something finished is not "in progress".
    pub fn is_resumable(&self) -> bool {
        !self.played
            && self.position_ticks > 0
            && self
                .played_percentage()
                .is_some_and(|percent| percent > 1.0)
    }
}

#[derive(Debug, Clone, Default)]
struct ProfileSnapshot {
    items: HashMap<String, PlaybackState>,
    favorites: HashSet<String>,
    /// Most recently updated first, so Resume needs no re-sorting.
    recent: Vec<String>,
}

static SNAPSHOTS: Lazy<RwLock<HashMap<String, (Instant, ProfileSnapshot)>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static LAST_WRITE: Lazy<RwLock<HashMap<String, Instant>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Profile ids are `preferences.id`, a UUID. Anything else is refused rather
/// than interpolated into a PostgREST filter.
fn is_profile_id(value: &str) -> bool {
    let stripped: Vec<char> = value.chars().filter(|c| *c != '-').collect();
    stripped.len() == 32 && stripped.iter().all(char::is_ascii_hexdigit)
}

fn supabase() -> Option<(String, String)> {
    let endpoint = std::env::var("SUPABASE_URL").ok()?;
    let key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").ok()?;
    (!endpoint.is_empty() && !key.is_empty()).then_some((endpoint, key))
}

// ---------------------------------------------------------------------------
// Reads
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PlaystateRow {
    item_id: String,
    #[serde(default)]
    position_ticks: i64,
    #[serde(default)]
    runtime_ticks: Option<i64>,
    #[serde(default)]
    played: bool,
    #[serde(default)]
    play_count: i32,
}

#[derive(Debug, Deserialize)]
struct FavoriteRow {
    item_id: String,
}

async fn fetch_snapshot(profile_id: &str) -> ProfileSnapshot {
    let Some((endpoint, key)) = supabase() else {
        return ProfileSnapshot::default();
    };
    let client = crate::engines::http::client();

    let states = client
        .get(format!("{endpoint}/rest/v1/playstate"))
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {key}"))
        .query(&[
            ("profile_id", format!("eq.{profile_id}").as_str()),
            ("order", "updated_at.desc"),
            ("limit", "500"),
        ])
        .send()
        .await;

    let favorites = client
        .get(format!("{endpoint}/rest/v1/favorites"))
        .header("apikey", &key)
        .header("Authorization", format!("Bearer {key}"))
        .query(&[
            ("profile_id", format!("eq.{profile_id}").as_str()),
            ("order", "created_at.desc"),
            ("limit", "500"),
        ])
        .send()
        .await;

    let mut snapshot = ProfileSnapshot::default();

    if let Ok(response) = states {
        if let Ok(rows) = response.json::<Vec<PlaystateRow>>().await {
            for row in rows {
                snapshot.recent.push(row.item_id.clone());
                snapshot.items.insert(
                    row.item_id,
                    PlaybackState {
                        position_ticks: row.position_ticks,
                        runtime_ticks: row.runtime_ticks,
                        played: row.played,
                        play_count: row.play_count,
                        is_favorite: false,
                    },
                );
            }
        }
    }

    if let Ok(response) = favorites {
        if let Ok(rows) = response.json::<Vec<FavoriteRow>>().await {
            for row in rows {
                snapshot.favorites.insert(row.item_id);
            }
        }
    }

    for (item_id, state) in snapshot.items.iter_mut() {
        state.is_favorite = snapshot.favorites.contains(item_id);
    }

    snapshot
}

async fn snapshot_for(profile_id: &str) -> ProfileSnapshot {
    if let Ok(cache) = SNAPSHOTS.read() {
        if let Some((fetched_at, snapshot)) = cache.get(profile_id) {
            if fetched_at.elapsed() < SNAPSHOT_TTL {
                return snapshot.clone();
            }
        }
    }

    let snapshot = fetch_snapshot(profile_id).await;
    if let Ok(mut cache) = SNAPSHOTS.write() {
        cache.insert(profile_id.to_string(), (Instant::now(), snapshot.clone()));
    }
    snapshot
}

/// State for one item, for an item page.
pub async fn state_for(profile_id: &str, item_id: &str) -> PlaybackState {
    if !is_profile_id(profile_id) {
        return PlaybackState::default();
    }

    let snapshot = snapshot_for(profile_id).await;
    let mut state = snapshot.items.get(item_id).cloned().unwrap_or_default();
    state.is_favorite = snapshot.favorites.contains(item_id);
    state
}

/// State for a whole page of items at once, so a listing costs one snapshot
/// rather than a query per row.
pub async fn states_for(profile_id: &str, item_ids: &[String]) -> HashMap<String, PlaybackState> {
    if !is_profile_id(profile_id) {
        return HashMap::new();
    }

    let snapshot = snapshot_for(profile_id).await;
    item_ids
        .iter()
        .map(|item_id| {
            let mut state = snapshot.items.get(item_id).cloned().unwrap_or_default();
            state.is_favorite = snapshot.favorites.contains(item_id);
            (item_id.clone(), state)
        })
        .collect()
}

/// Part-watched items, most recent first.
pub async fn resumable(profile_id: &str, limit: usize) -> Vec<String> {
    if !is_profile_id(profile_id) {
        return Vec::new();
    }

    let snapshot = snapshot_for(profile_id).await;
    snapshot
        .recent
        .iter()
        .filter(|item_id| {
            snapshot
                .items
                .get(*item_id)
                .is_some_and(PlaybackState::is_resumable)
        })
        .take(limit)
        .cloned()
        .collect()
}

/// Everything with recorded state, most recently touched first, paired with it.
/// Up Next reads this to work out where each series was left.
pub async fn recent_items(profile_id: &str, limit: usize) -> Vec<(String, PlaybackState)> {
    if !is_profile_id(profile_id) {
        return Vec::new();
    }

    let snapshot = snapshot_for(profile_id).await;
    snapshot
        .recent
        .iter()
        .filter_map(|item_id| {
            snapshot
                .items
                .get(item_id)
                .map(|state| (item_id.clone(), state.clone()))
        })
        .take(limit)
        .collect()
}

pub async fn favorite_items(profile_id: &str, limit: usize) -> Vec<String> {
    if !is_profile_id(profile_id) {
        return Vec::new();
    }

    snapshot_for(profile_id)
        .await
        .favorites
        .into_iter()
        .take(limit)
        .collect()
}

// ---------------------------------------------------------------------------
// Writes
// ---------------------------------------------------------------------------

/// Applies a change to the cached snapshot straight away, so the next read
/// reflects it even though the database write happens behind the request.
fn update_snapshot(profile_id: &str, item_id: &str, apply: impl FnOnce(&mut ProfileSnapshot)) {
    let Ok(mut cache) = SNAPSHOTS.write() else {
        return;
    };
    let entry = cache
        .entry(profile_id.to_string())
        .or_insert_with(|| (Instant::now(), ProfileSnapshot::default()));

    apply(&mut entry.1);

    // Touched items lead the recency list, which is what Resume reads.
    entry.1.recent.retain(|existing| existing != item_id);
    entry.1.recent.insert(0, item_id.to_string());
}

fn should_write(profile_id: &str, item_id: &str) -> bool {
    let key = format!("{profile_id}:{item_id}");
    let Ok(mut last) = LAST_WRITE.write() else {
        return true;
    };

    let now = Instant::now();
    last.retain(|_, written| now.duration_since(*written) < Duration::from_secs(3600));

    match last.get(&key) {
        Some(written) if now.duration_since(*written) < WRITE_DEBOUNCE => false,
        _ => {
            last.insert(key, now);
            true
        }
    }
}

/// Remembers how long an item runs.
///
/// Called when sources are resolved, because progress reports do not carry a
/// runtime and without one there is no way to tell "ten minutes in" from
/// "finished". In-process only: it reaches the database with the first progress
/// write, and is re-established by the next `PlaybackInfo` after a restart.
pub fn note_runtime(profile_id: &str, item_id: &str, runtime_ticks: Option<i64>) {
    let Some(runtime_ticks) = runtime_ticks.filter(|ticks| *ticks > 0) else {
        return;
    };
    if !is_profile_id(profile_id) {
        return;
    }

    update_snapshot(profile_id, item_id, |snapshot| {
        snapshot
            .items
            .entry(item_id.to_string())
            .or_default()
            .runtime_ticks = Some(runtime_ticks);
    });
}

/// Records how far into an item a viewer has got.
///
/// Never blocks the caller: clients report progress every few seconds and a
/// slow database must not become a slow player.
pub fn record_progress(
    profile_id: &str,
    item_id: &str,
    atlas_key: &str,
    position_ticks: i64,
    runtime_ticks: Option<i64>,
) {
    if !is_profile_id(profile_id) {
        return;
    }

    let mut effective_runtime = runtime_ticks;
    let mut played = false;

    update_snapshot(profile_id, item_id, |snapshot| {
        let state = snapshot.items.entry(item_id.to_string()).or_default();
        state.position_ticks = position_ticks;
        if runtime_ticks.is_some() {
            state.runtime_ticks = runtime_ticks;
        }

        // Fall back to a runtime learned earlier in the session, since the
        // report itself does not carry one.
        effective_runtime = state.runtime_ticks;
        played = effective_runtime
            .filter(|ticks| *ticks > 0)
            .is_some_and(|ticks| position_ticks as f64 / ticks as f64 >= PLAYED_FRACTION);

        if played && !state.played {
            state.played = true;
            state.play_count += 1;
        }
    });

    // A finish is worth persisting immediately; intermediate ticks are not.
    if !played && !should_write(profile_id, item_id) {
        return;
    }

    persist_playstate(
        profile_id.to_string(),
        item_id.to_string(),
        atlas_key.to_string(),
        position_ticks,
        effective_runtime,
        played,
    );
}

/// Explicitly marking an item watched or unwatched.
pub fn set_played(profile_id: &str, item_id: &str, atlas_key: &str, played: bool) {
    if !is_profile_id(profile_id) {
        return;
    }

    update_snapshot(profile_id, item_id, |snapshot| {
        let state = snapshot.items.entry(item_id.to_string()).or_default();
        state.played = played;
        state.position_ticks = 0;
        if played {
            state.play_count += 1;
        }
    });

    persist_playstate(
        profile_id.to_string(),
        item_id.to_string(),
        atlas_key.to_string(),
        0,
        None,
        played,
    );
}

pub fn set_favorite(profile_id: &str, item_id: &str, atlas_key: &str, favorite: bool) {
    if !is_profile_id(profile_id) {
        return;
    }

    update_snapshot(profile_id, item_id, |snapshot| {
        if favorite {
            snapshot.favorites.insert(item_id.to_string());
        } else {
            snapshot.favorites.remove(item_id);
        }
        if let Some(state) = snapshot.items.get_mut(item_id) {
            state.is_favorite = favorite;
        }
    });

    let (profile_id, item_id, atlas_key) = (
        profile_id.to_string(),
        item_id.to_string(),
        atlas_key.to_string(),
    );

    tokio::spawn(async move {
        let Some((endpoint, key)) = supabase() else {
            return;
        };
        let client = crate::engines::http::client();

        let result = if favorite {
            client
                .post(format!("{endpoint}/rest/v1/favorites"))
                .header("apikey", &key)
                .header("Authorization", format!("Bearer {key}"))
                .header("Prefer", "resolution=merge-duplicates")
                .json(&serde_json::json!({
                    "profile_id": profile_id,
                    "item_id": item_id,
                    "atlas_key": atlas_key,
                }))
                .send()
                .await
        } else {
            client
                .delete(format!("{endpoint}/rest/v1/favorites"))
                .header("apikey", &key)
                .header("Authorization", format!("Bearer {key}"))
                .query(&[
                    ("profile_id", format!("eq.{profile_id}").as_str()),
                    ("item_id", format!("eq.{item_id}").as_str()),
                ])
                .send()
                .await
        };

        if let Err(error) = result {
            tracing::warn!(%error, "failed to persist favorite");
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn persist_playstate(
    profile_id: String,
    item_id: String,
    atlas_key: String,
    position_ticks: i64,
    runtime_ticks: Option<i64>,
    played: bool,
) {
    tokio::spawn(async move {
        let Some((endpoint, key)) = supabase() else {
            return;
        };

        let response = crate::engines::http::client()
            .post(format!("{endpoint}/rest/v1/playstate"))
            .header("apikey", &key)
            .header("Authorization", format!("Bearer {key}"))
            .header("Prefer", "resolution=merge-duplicates")
            .json(&serde_json::json!({
                "profile_id": profile_id,
                "item_id": item_id,
                "atlas_key": atlas_key,
                "position_ticks": position_ticks,
                "runtime_ticks": runtime_ticks,
                "played": played,
                "updated_at": chrono::Utc::now().to_rfc3339(),
            }))
            .send()
            .await;

        match response {
            Ok(response) if !response.status().is_success() => {
                tracing::warn!(status = %response.status(), "failed to persist playstate");
            }
            Err(error) => tracing::warn!(%error, "failed to persist playstate"),
            _ => {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{is_profile_id, PlaybackState, PLAYED_FRACTION};

    fn state(position: i64, runtime: i64) -> PlaybackState {
        PlaybackState {
            position_ticks: position,
            runtime_ticks: Some(runtime),
            ..PlaybackState::default()
        }
    }

    #[test]
    fn only_uuid_shaped_profiles_are_accepted() {
        // These reach a PostgREST filter, so anything unexpected is refused
        // rather than interpolated.
        assert!(is_profile_id("3f1a2b4c-5d6e-4f70-8192-a3b4c5d6e7f8"));
        assert!(is_profile_id("3f1a2b4c5d6e4f708192a3b4c5d6e7f8"));
        assert!(!is_profile_id("not-a-uuid"));
        assert!(!is_profile_id(""));
        assert!(!is_profile_id("eq.1&or=(true)"));
    }

    #[test]
    fn a_glance_at_something_is_not_worth_resuming() {
        // Ten seconds into a two-hour film is an accident, not a session.
        let barely_started = state(60_000_000, 72_000_000_000);

        assert!(!barely_started.is_resumable());
        assert!(state(30_000_000_000, 72_000_000_000).is_resumable());
        assert!(!state(0, 72_000_000_000).is_resumable());
    }

    #[test]
    fn a_finished_item_is_not_offered_for_resuming() {
        let mut finished = state(70_000_000_000, 72_000_000_000);
        finished.played = true;

        assert!(!finished.is_resumable());
    }

    #[test]
    fn progress_without_a_known_runtime_is_not_resumable() {
        // Percentage is meaningless without one, and guessing produces a resume
        // point in the wrong place.
        let unknown = PlaybackState {
            position_ticks: 30_000_000_000,
            runtime_ticks: None,
            ..PlaybackState::default()
        };

        assert_eq!(unknown.played_percentage(), None);
        assert!(!unknown.is_resumable());
    }

    #[test]
    fn played_percentage_is_clamped_to_the_runtime() {
        // Clients occasionally report a position past the end.
        let overrun = state(80_000_000_000, 72_000_000_000);

        assert_eq!(overrun.played_percentage(), Some(100.0));
    }

    #[test]
    fn the_played_threshold_leaves_the_credits_out() {
        assert_eq!(PLAYED_FRACTION, 0.9);

        let near_end = state(65_000_000_000, 72_000_000_000);
        assert!(near_end.played_percentage().unwrap() > 90.0);
    }
}
