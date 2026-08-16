//! Case-insensitive query parameters.
//!
//! Jellyfin treats query parameters case-insensitively and clients disagree
//! about casing — Infuse sends `ParentId` where others send `parentId`. Axum's
//! typed `Query<T>` matches exactly, so parameters are read through this
//! instead, in one place rather than at every handler.

use std::collections::HashMap;

/// Clients ask for enormous pages. Cinemeta pages in hundreds and every item
/// costs a hydrate, so the server decides the ceiling.
pub const MAX_LIMIT: usize = 100;
const DEFAULT_LIMIT: usize = 50;

#[derive(Debug, Clone, Default)]
pub struct JellyfinQuery {
    params: HashMap<String, String>,
}

impl JellyfinQuery {
    pub fn from_map(raw: HashMap<String, String>) -> Self {
        JellyfinQuery {
            params: raw
                .into_iter()
                .map(|(key, value)| (key.to_lowercase(), value))
                .collect(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.params
            .get(&name.to_lowercase())
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    pub fn number(&self, name: &str) -> Option<usize> {
        self.get(name)?.parse().ok()
    }

    pub fn flag(&self, name: &str) -> bool {
        matches!(
            self.get(name).map(str::to_ascii_lowercase).as_deref(),
            Some("true" | "1")
        )
    }

    /// A comma-separated parameter such as `IncludeItemTypes=Movie,Series`.
    pub fn list(&self, name: &str) -> Vec<String> {
        self.get(name)
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|part| !part.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn start_index(&self) -> usize {
        self.number("StartIndex").unwrap_or(0)
    }

    /// Always bounded, whatever the client asked for.
    pub fn limit(&self) -> usize {
        self.number("Limit")
            .unwrap_or(DEFAULT_LIMIT)
            .clamp(1, MAX_LIMIT)
    }

    pub fn search_term(&self) -> Option<&str> {
        self.get("SearchTerm")
            .map(str::trim)
            .filter(|term| !term.is_empty())
    }

    pub fn parent_id(&self) -> Option<&str> {
        self.get("ParentId")
    }

    /// Whether the client filtered to types Atlas cannot serve at all — music,
    /// photos, live TV. The honest answer there is an empty page rather than
    /// films relabelled as albums.
    pub fn wants_none_of(&self, served: &[&str]) -> bool {
        let requested = self.list("IncludeItemTypes");
        if requested.is_empty() {
            return false;
        }
        !requested.iter().any(|kind| {
            served
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(kind))
        })
    }

    pub fn includes_type(&self, kind: &str) -> bool {
        let requested = self.list("IncludeItemTypes");
        requested.is_empty()
            || requested
                .iter()
                .any(|value| value.eq_ignore_ascii_case(kind))
    }
}

#[cfg(test)]
mod tests {
    use super::{JellyfinQuery, MAX_LIMIT};
    use std::collections::HashMap;

    fn query(pairs: &[(&str, &str)]) -> JellyfinQuery {
        JellyfinQuery::from_map(
            pairs
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<HashMap<_, _>>(),
        )
    }

    #[test]
    fn parameters_match_whatever_casing_a_client_uses() {
        // Infuse sends ParentId; other clients send parentId.
        for key in ["ParentId", "parentId", "PARENTID"] {
            assert_eq!(query(&[(key, "abc")]).parent_id(), Some("abc"));
        }
    }

    #[test]
    fn empty_values_read_as_absent() {
        assert_eq!(query(&[("SearchTerm", "")]).search_term(), None);
        assert_eq!(query(&[("SearchTerm", "   ")]).search_term(), None);
    }

    #[test]
    fn the_page_size_is_the_servers_decision() {
        assert_eq!(query(&[("Limit", "20")]).limit(), 20);
        assert_eq!(query(&[("Limit", "100000")]).limit(), MAX_LIMIT);
        assert_eq!(query(&[("Limit", "0")]).limit(), 1);
        assert_eq!(query(&[("Limit", "nonsense")]).limit(), 50);
        assert_eq!(query(&[]).limit(), 50);
    }

    #[test]
    fn start_index_defaults_to_the_beginning() {
        assert_eq!(query(&[]).start_index(), 0);
        assert_eq!(query(&[("startIndex", "40")]).start_index(), 40);
    }

    #[test]
    fn item_type_filters_are_split_and_compared_loosely() {
        let request = query(&[("IncludeItemTypes", "Movie,Series")]);

        assert!(request.includes_type("movie"));
        assert!(request.includes_type("Series"));
        assert!(!request.includes_type("MusicAlbum"));
        // No filter at all means the client will take anything.
        assert!(query(&[]).includes_type("MusicAlbum"));
    }

    #[test]
    fn recognises_a_request_for_types_atlas_does_not_carry() {
        let served = ["Movie", "Series", "Season", "Episode"];

        assert!(query(&[("IncludeItemTypes", "MusicAlbum,Audio")]).wants_none_of(&served));
        // A mixed request is still servable in part, so it is not refused.
        assert!(!query(&[("IncludeItemTypes", "Movie,MusicAlbum")]).wants_none_of(&served));
        assert!(!query(&[("IncludeItemTypes", "Movie")]).wants_none_of(&served));
        // No filter means the client will take whatever it is given.
        assert!(!query(&[]).wants_none_of(&served));
    }

    #[test]
    fn flags_accept_the_spellings_clients_send() {
        assert!(query(&[("Recursive", "true")]).flag("recursive"));
        assert!(query(&[("Recursive", "True")]).flag("Recursive"));
        assert!(query(&[("Recursive", "1")]).flag("recursive"));
        assert!(!query(&[("Recursive", "false")]).flag("recursive"));
        assert!(!query(&[]).flag("recursive"));
    }
}
