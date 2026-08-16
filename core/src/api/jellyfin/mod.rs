//! A Jellyfin-compatible surface, scoped to what Infuse's Direct Mode asks for.
//!
//! Infuse speaks the Jellyfin/Emby API natively, and in Direct Mode it fetches
//! on demand rather than syncing a library up front — the only mode that suits a
//! server with no library of its own. Nothing here is mounted unless
//! `ATLAS_JELLYFIN_ENABLED` is set, so the Stremio surface is unaffected by
//! default.

pub mod auth;
pub mod dto;
pub mod ids;
pub mod sessions;
pub mod system;
pub mod trace;
pub mod ua;
pub mod users;

use axum::Router;

/// Whether to mount the Jellyfin surface at all.
pub fn enabled() -> bool {
    flag("ATLAS_JELLYFIN_ENABLED")
}

/// Whether unmatched routes should be logged and answered permissively instead
/// of 404ing. Used to discover what a real client asks for; off in production.
pub fn permissive() -> bool {
    flag("ATLAS_JELLYFIN_PERMISSIVE")
}

fn flag(name: &str) -> bool {
    std::env::var(name)
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes"))
        .unwrap_or(false)
}

/// A stable, non-reversible 32-hex identifier.
///
/// Jellyfin renders GUIDs as 32 undashed hex characters, and clients treat a
/// changed id as a different server or user. Two FNV-1a passes over distinct
/// orderings give 128 deterministic bits without holding on to the input.
pub fn stable_hex_id(namespace: &str, value: &str) -> String {
    let forward = fnv1a(&format!("{namespace}:{value}"));
    let reverse = fnv1a(&format!("{value}:{namespace}"));
    format!("{forward:016x}{reverse:016x}")
}

fn fnv1a(value: &str) -> u64 {
    let mut hash: u64 = 14_695_981_039_346_656_037;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

/// Derived from the public base URL so staging and production never claim the
/// same server identity in a client that has both configured.
pub fn server_id() -> String {
    let base = std::env::var("ATLAS_PUBLIC_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    stable_hex_id("atlas-server", &base)
}

/// `None` when the surface is disabled, so `main` mounts no routes at all
/// rather than mounting routes that refuse to answer.
pub fn router() -> Option<Router> {
    if !enabled() {
        tracing::info!("Jellyfin surface disabled (set ATLAS_JELLYFIN_ENABLED=1 to mount it)");
        return None;
    }

    tracing::info!(permissive = permissive(), "Jellyfin surface enabled");

    let routes = Router::new()
        .merge(system::router())
        .merge(users::router())
        .merge(sessions::router())
        .fallback(trace::unmatched);

    // Official Jellyfin answers on both prefixes, and clients configured against
    // an Emby-style URL will use the second.
    Some(
        Router::new()
            .nest("/jellyfin", routes.clone())
            .nest("/emby", routes),
    )
}

#[cfg(test)]
mod tests {
    use super::{server_id, stable_hex_id};

    #[test]
    fn stable_ids_are_guid_shaped_and_deterministic() {
        let first = stable_hex_id("atlas-user", "token-abc");
        let second = stable_hex_id("atlas-user", "token-abc");

        assert_eq!(first, second);
        assert_eq!(first.len(), 32);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn stable_ids_separate_namespaces_and_values() {
        assert_ne!(
            stable_hex_id("atlas-user", "token-abc"),
            stable_hex_id("atlas-server", "token-abc")
        );
        assert_ne!(
            stable_hex_id("atlas-user", "token-abc"),
            stable_hex_id("atlas-user", "token-xyz")
        );
    }

    #[test]
    fn the_server_id_does_not_leak_the_base_url() {
        let id = server_id();

        assert_eq!(id.len(), 32);
        assert!(!id.contains("127.0.0.1"));
    }
}
