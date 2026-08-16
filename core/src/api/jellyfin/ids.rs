//! Jellyfin item ids.
//!
//! Jellyfin identifies every item with a GUID, rendered in its API as 32
//! undashed hex characters. Atlas has no item database to allocate ids from, so
//! ids are a reversible encoding of the thing they name: 16 bytes packed into
//! that same 32-hex shape. Nothing is stored, and the same title yields the same
//! id on every machine and across restarts — Infuse warns about items whose ids
//! move.
//!
//! `AtlasID` alone is not enough to name an item. `AtlasID::IMDb { season: None,
//! episode: None }` describes both a film and a series root, and a season has no
//! representation at all, so the encoding carries an explicit item kind.
//!
//! ```text
//! byte  0     magic 0xA7
//! byte  1     kind
//! byte  2     flags  bit0 namespace (0 = IMDb, 1 = TMDB)
//!                    bit1 season present
//!                    bit2 episode present
//! byte  3     reserved
//! bytes 4-5   season   u16 BE
//! bytes 6-7   episode  u16 BE
//! bytes 8-15  payload  u64 BE — IMDb number, TMDB id, or a library slug
//! ```
//!
//! Season and episode presence are flags rather than "zero means absent",
//! because zero is a real value: Cinemeta emits season 0 for specials, and
//! folding that into the series node would hide them.

use crate::engines::identity::AtlasID;

const MAGIC: u8 = 0xA7;

const KIND_ROOT: u8 = 0x00;
const KIND_LIBRARY: u8 = 0x01;
const KIND_MOVIE: u8 = 0x02;
const KIND_SERIES: u8 = 0x03;
const KIND_SEASON: u8 = 0x04;
const KIND_EPISODE: u8 = 0x05;

const FLAG_TMDB: u8 = 0b0000_0001;
const FLAG_HAS_SEASON: u8 = 0b0000_0010;
const FLAG_HAS_EPISODE: u8 = 0b0000_0100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Root,
    Library,
    Movie,
    Series,
    Season,
    Episode,
}

impl ItemKind {
    fn tag(self) -> u8 {
        match self {
            ItemKind::Root => KIND_ROOT,
            ItemKind::Library => KIND_LIBRARY,
            ItemKind::Movie => KIND_MOVIE,
            ItemKind::Series => KIND_SERIES,
            ItemKind::Season => KIND_SEASON,
            ItemKind::Episode => KIND_EPISODE,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            KIND_ROOT => Some(ItemKind::Root),
            KIND_LIBRARY => Some(ItemKind::Library),
            KIND_MOVIE => Some(ItemKind::Movie),
            KIND_SERIES => Some(ItemKind::Series),
            KIND_SEASON => Some(ItemKind::Season),
            KIND_EPISODE => Some(ItemKind::Episode),
            _ => None,
        }
    }

    /// The `Type` string Jellyfin clients switch on.
    pub fn type_name(self) -> &'static str {
        match self {
            ItemKind::Root => "Folder",
            ItemKind::Library => "CollectionFolder",
            ItemKind::Movie => "Movie",
            ItemKind::Series => "Series",
            ItemKind::Season => "Season",
            ItemKind::Episode => "Episode",
        }
    }
}

/// The two libraries Atlas presents. Slugs are eight bytes so they pack into the
/// payload field whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Library {
    Movies,
    Shows,
}

impl Library {
    const MOVIES_SLUG: u64 = u64::from_be_bytes(*b"movies__");
    const SHOWS_SLUG: u64 = u64::from_be_bytes(*b"tvshows_");

    fn slug(self) -> u64 {
        match self {
            Library::Movies => Self::MOVIES_SLUG,
            Library::Shows => Self::SHOWS_SLUG,
        }
    }

    fn from_slug(slug: u64) -> Option<Self> {
        match slug {
            Self::MOVIES_SLUG => Some(Library::Movies),
            Self::SHOWS_SLUG => Some(Library::Shows),
            _ => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Library::Movies => "Movies",
            Library::Shows => "TV Shows",
        }
    }

    /// Jellyfin's `CollectionType`. Getting this wrong makes clients apply the
    /// film metadata agent to series.
    pub fn collection_type(self) -> &'static str {
        match self {
            Library::Movies => "movies",
            Library::Shows => "tvshows",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Imdb,
    Tmdb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemId {
    pub kind: ItemKind,
    pub namespace: Namespace,
    pub season: Option<u16>,
    pub episode: Option<u16>,
    pub payload: u64,
}

impl ItemId {
    pub fn root() -> Self {
        ItemId {
            kind: ItemKind::Root,
            namespace: Namespace::Imdb,
            season: None,
            episode: None,
            payload: 0,
        }
    }

    pub fn library(library: Library) -> Self {
        ItemId {
            kind: ItemKind::Library,
            namespace: Namespace::Imdb,
            season: None,
            episode: None,
            payload: library.slug(),
        }
    }

    /// The library this id belongs to, for `Library` ids.
    pub fn as_library(&self) -> Option<Library> {
        (self.kind == ItemKind::Library)
            .then(|| Library::from_slug(self.payload))
            .flatten()
    }

    /// Encode a playable item. An `AtlasID` carrying a season and episode is an
    /// episode; anything else is treated as a film, matching
    /// `AtlasID::media_type`.
    pub fn from_atlas_id(atlas_id: &AtlasID) -> Self {
        match atlas_id {
            AtlasID::IMDb {
                id,
                season: Some(season),
                episode: Some(episode),
            } => ItemId {
                kind: ItemKind::Episode,
                namespace: Namespace::Imdb,
                season: Some(*season as u16),
                episode: Some(*episode as u16),
                payload: imdb_number(id),
            },
            AtlasID::IMDb { id, .. } => ItemId {
                kind: ItemKind::Movie,
                namespace: Namespace::Imdb,
                season: None,
                episode: None,
                payload: imdb_number(id),
            },
            AtlasID::TMDB(id) => ItemId {
                kind: ItemKind::Movie,
                namespace: Namespace::Tmdb,
                season: None,
                episode: None,
                payload: u64::from(*id),
            },
        }
    }

    pub fn series(namespace: Namespace, payload: u64) -> Self {
        ItemId {
            kind: ItemKind::Series,
            namespace,
            season: None,
            episode: None,
            payload,
        }
    }

    pub fn season(namespace: Namespace, payload: u64, season: u16) -> Self {
        ItemId {
            kind: ItemKind::Season,
            namespace,
            season: Some(season),
            episode: None,
            payload,
        }
    }

    pub fn episode(namespace: Namespace, payload: u64, season: u16, episode: u16) -> Self {
        ItemId {
            kind: ItemKind::Episode,
            namespace,
            season: Some(season),
            episode: Some(episode),
            payload,
        }
    }

    /// The series this item belongs to, for seasons and episodes.
    pub fn series_id(&self) -> Option<ItemId> {
        matches!(self.kind, ItemKind::Season | ItemKind::Episode)
            .then(|| ItemId::series(self.namespace, self.payload))
    }

    /// The season this episode belongs to.
    pub fn season_id(&self) -> Option<ItemId> {
        match (self.kind, self.season) {
            (ItemKind::Episode, Some(season)) => {
                Some(ItemId::season(self.namespace, self.payload, season))
            }
            _ => None,
        }
    }

    /// The node directly above this one. Films and series sit under a library;
    /// the root is its own parent.
    pub fn parent_id(&self) -> ItemId {
        match self.kind {
            ItemKind::Root => ItemId::root(),
            ItemKind::Library => ItemId::root(),
            ItemKind::Movie => ItemId::library(Library::Movies),
            ItemKind::Series => ItemId::library(Library::Shows),
            ItemKind::Season => self.series_id().unwrap_or_else(ItemId::root),
            ItemKind::Episode => self.season_id().unwrap_or_else(ItemId::root),
        }
    }

    /// The IMDb id this item hangs off, for metadata lookups. Available for
    /// series and seasons too, which are not themselves playable.
    pub fn imdb_id(&self) -> Option<String> {
        (self.namespace == Namespace::Imdb
            && !matches!(self.kind, ItemKind::Root | ItemKind::Library))
        .then(|| format!("tt{:07}", self.payload))
    }

    /// The `AtlasID` to resolve sources for.
    ///
    /// `None` for series and seasons: they are navigational, and returning
    /// `None` is what stops a `PlaybackInfo` request against a node that has no
    /// stream of its own.
    pub fn to_playable_atlas_id(&self) -> Option<AtlasID> {
        match (self.kind, self.namespace) {
            (ItemKind::Movie, Namespace::Imdb) => Some(AtlasID::IMDb {
                id: self.imdb_id()?,
                season: None,
                episode: None,
            }),
            (ItemKind::Movie, Namespace::Tmdb) => Some(AtlasID::TMDB(self.payload as u32)),
            (ItemKind::Episode, Namespace::Imdb) => Some(AtlasID::IMDb {
                id: self.imdb_id()?,
                season: Some(u32::from(self.season?)),
                episode: Some(u32::from(self.episode?)),
            }),
            _ => None,
        }
    }

    pub fn to_hex(self) -> String {
        let mut flags = 0u8;
        if self.namespace == Namespace::Tmdb {
            flags |= FLAG_TMDB;
        }
        if self.season.is_some() {
            flags |= FLAG_HAS_SEASON;
        }
        if self.episode.is_some() {
            flags |= FLAG_HAS_EPISODE;
        }

        let mut bytes = [0u8; 16];
        bytes[0] = MAGIC;
        bytes[1] = self.kind.tag();
        bytes[2] = flags;
        bytes[4..6].copy_from_slice(&self.season.unwrap_or(0).to_be_bytes());
        bytes[6..8].copy_from_slice(&self.episode.unwrap_or(0).to_be_bytes());
        bytes[8..16].copy_from_slice(&self.payload.to_be_bytes());

        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// Decode an id from a client. Dashes and case are tolerated because clients
    /// are inconsistent about GUID formatting.
    pub fn parse(value: &str) -> Option<ItemId> {
        let cleaned: String = value.chars().filter(|c| *c != '-').collect();
        if cleaned.len() != 32 {
            return None;
        }

        let mut bytes = [0u8; 16];
        for (index, byte) in bytes.iter_mut().enumerate() {
            let start = index * 2;
            *byte = u8::from_str_radix(cleaned.get(start..start + 2)?, 16).ok()?;
        }

        if bytes[0] != MAGIC {
            return None;
        }

        let kind = ItemKind::from_tag(bytes[1])?;
        let flags = bytes[2];
        let namespace = if flags & FLAG_TMDB != 0 {
            Namespace::Tmdb
        } else {
            Namespace::Imdb
        };

        let season =
            (flags & FLAG_HAS_SEASON != 0).then(|| u16::from_be_bytes([bytes[4], bytes[5]]));
        let episode =
            (flags & FLAG_HAS_EPISODE != 0).then(|| u16::from_be_bytes([bytes[6], bytes[7]]));
        let payload = u64::from_be_bytes(bytes[8..16].try_into().ok()?);

        Some(ItemId {
            kind,
            namespace,
            season,
            episode,
            payload,
        })
    }
}

/// "tt0944947" carries its identity in the digits; the prefix is constant.
fn imdb_number(imdb_id: &str) -> u64 {
    imdb_id
        .trim_start_matches("tt")
        .parse::<u64>()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{ItemId, Library, Namespace};
    use crate::engines::identity::AtlasID;

    fn round_trip(id: ItemId) -> ItemId {
        let hex = id.to_hex();
        assert_eq!(hex.len(), 32, "ids must be 32 hex characters");
        ItemId::parse(&hex).expect("an id Atlas produced must parse back")
    }

    #[test]
    fn round_trips_every_item_kind() {
        for id in [
            ItemId::root(),
            ItemId::library(Library::Movies),
            ItemId::library(Library::Shows),
            ItemId::from_atlas_id(&AtlasID::IMDb {
                id: "tt0133093".to_string(),
                season: None,
                episode: None,
            }),
            ItemId::series(Namespace::Imdb, 944_947),
            ItemId::season(Namespace::Imdb, 944_947, 1),
            ItemId::episode(Namespace::Imdb, 944_947, 1, 2),
        ] {
            assert_eq!(round_trip(id), id);
        }
    }

    #[test]
    fn season_zero_stays_distinct_from_a_series() {
        // Cinemeta uses season 0 for specials, so "absent" cannot be encoded as
        // zero without hiding them behind the series node.
        let specials = ItemId::season(Namespace::Imdb, 944_947, 0);
        let series = ItemId::series(Namespace::Imdb, 944_947);

        assert_ne!(specials.to_hex(), series.to_hex());
        assert_eq!(round_trip(specials).season, Some(0));
        assert_eq!(round_trip(series).season, None);
    }

    #[test]
    fn episode_zero_survives_the_round_trip() {
        let episode = ItemId::episode(Namespace::Imdb, 944_947, 1, 0);

        assert_eq!(round_trip(episode).episode, Some(0));
    }

    #[test]
    fn a_film_and_a_series_with_one_imdb_id_get_different_ids() {
        let film = ItemId::from_atlas_id(&AtlasID::IMDb {
            id: "tt0944947".to_string(),
            season: None,
            episode: None,
        });
        let series = ItemId::series(Namespace::Imdb, 944_947);

        assert_ne!(film.to_hex(), series.to_hex());
    }

    #[test]
    fn handles_long_running_shows_and_eight_digit_ids() {
        // One Piece is past episode 1000, and eight-digit IMDb ids are routine.
        let episode = ItemId::episode(Namespace::Imdb, 10_919_420, 21, 1050);
        let decoded = round_trip(episode);

        assert_eq!(decoded.episode, Some(1050));
        assert_eq!(decoded.imdb_id(), Some("tt10919420".to_string()));
    }

    #[test]
    fn pads_short_imdb_ids_back_to_their_original_form() {
        let film = ItemId::from_atlas_id(&AtlasID::IMDb {
            id: "tt0133093".to_string(),
            season: None,
            episode: None,
        });

        assert_eq!(film.imdb_id(), Some("tt0133093".to_string()));
    }

    #[test]
    fn round_trips_tmdb_ids() {
        let film = ItemId::from_atlas_id(&AtlasID::TMDB(550));
        let decoded = round_trip(film);

        assert_eq!(decoded.namespace, Namespace::Tmdb);
        assert_eq!(decoded.to_playable_atlas_id(), Some(AtlasID::TMDB(550)));
    }

    #[test]
    fn episodes_derive_their_season_and_series_without_a_lookup() {
        let episode = ItemId::episode(Namespace::Imdb, 944_947, 1, 2);

        assert_eq!(
            episode.series_id().map(ItemId::to_hex),
            Some(ItemId::series(Namespace::Imdb, 944_947).to_hex())
        );
        assert_eq!(
            episode.season_id().map(ItemId::to_hex),
            Some(ItemId::season(Namespace::Imdb, 944_947, 1).to_hex())
        );
        assert_eq!(
            episode.parent_id().to_hex(),
            episode.season_id().unwrap().to_hex()
        );
    }

    #[test]
    fn only_playable_kinds_yield_an_atlas_id() {
        // This is the guard that stops PlaybackInfo against a navigational node.
        assert!(ItemId::series(Namespace::Imdb, 944_947)
            .to_playable_atlas_id()
            .is_none());
        assert!(ItemId::season(Namespace::Imdb, 944_947, 1)
            .to_playable_atlas_id()
            .is_none());
        assert!(ItemId::library(Library::Movies)
            .to_playable_atlas_id()
            .is_none());

        let episode = ItemId::episode(Namespace::Imdb, 944_947, 1, 2);
        assert_eq!(
            episode.to_playable_atlas_id(),
            Some(AtlasID::IMDb {
                id: "tt0944947".to_string(),
                season: Some(1),
                episode: Some(2),
            })
        );
    }

    #[test]
    fn tolerates_dashes_and_uppercase() {
        let id = ItemId::episode(Namespace::Imdb, 944_947, 1, 2);
        let hex = id.to_hex();
        let dashed = format!(
            "{}-{}-{}-{}-{}",
            &hex[0..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..32]
        );

        assert_eq!(ItemId::parse(&dashed), Some(id));
        assert_eq!(ItemId::parse(&hex.to_uppercase()), Some(id));
    }

    #[test]
    fn rejects_ids_atlas_did_not_produce() {
        // A real Jellyfin GUID has no magic byte, and must not be mistaken for
        // an Atlas id and decoded into some arbitrary title.
        assert_eq!(ItemId::parse("f137a2dd21bbc1b99aa5c0f6bf02a805"), None);
        assert_eq!(ItemId::parse(""), None);
        assert_eq!(ItemId::parse("a7"), None);

        let mut short = ItemId::root().to_hex();
        short.pop();
        assert_eq!(ItemId::parse(&short), None);
    }

    #[test]
    fn libraries_decode_back_to_themselves() {
        assert_eq!(
            round_trip(ItemId::library(Library::Shows)).as_library(),
            Some(Library::Shows)
        );
        assert_eq!(ItemId::root().as_library(), None);
    }
}
