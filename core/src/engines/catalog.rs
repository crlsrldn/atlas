//! The browsable catalogue.
//!
//! Atlas has no library, so the shelves a client browses come from Cinemeta's
//! catalogue endpoints. Two rules shape everything here.
//!
//! **Browsing never resolves.** `engines::metadata::get_metadata` fetches
//! Cinemeta *and* Torrentio together, because it exists to produce streams. A
//! browse screen showing fifty tiles must not fire fifty Torrentio searches, so
//! this module talks to Cinemeta alone and never touches the provider path.
//!
//! **Catalogue traffic stays off the shared machinery.** It uses its own HTTP
//! client, so a slow Cinemeta cannot exhaust the connection pool that TorBox
//! resolution shares, and it caches in Redis rather than `engines::cache` —
//! that one is a single global mutex whose reads clone under the lock, and it
//! is on the Stremio resolve path.

use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CINEMETA: &str = "https://v3-cinemeta.strem.io";

/// Artwork is deterministic from an IMDb id, so posters need no catalogue
/// lookup and no bytes through Atlas.
const METAHUB: &str = "https://images.metahub.space";

/// How long a page is served before a refresh is triggered behind it.
const SOFT_TTL: Duration = Duration::from_secs(30 * 60);
/// How long a page may be served at all. Generous: a stale shelf is a far
/// better failure than an empty one.
const HARD_TTL: Duration = Duration::from_secs(6 * 60 * 60);

/// Verified against Cinemeta: every catalogue response carries 50 entries, and
/// a short page is the signal that upstream has run out.
const CINEMETA_PAGE: usize = 50;

/// Deliberately separate from `engines::http`. Catalogue fan-out is bursty and
/// must not compete with resolution for connections.
static CATALOG_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(12))
        .pool_max_idle_per_host(8)
        .build()
        .expect("failed to build catalog HTTP client")
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Movie,
    Series,
}

impl MediaKind {
    pub fn slug(self) -> &'static str {
        match self {
            MediaKind::Movie => "movie",
            MediaKind::Series => "series",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogKind {
    Popular,
    New,
    Featured,
}

impl CatalogKind {
    fn slug(self) -> &'static str {
        match self {
            CatalogKind::Popular => "top",
            CatalogKind::New => "year",
            CatalogKind::Featured => "imdbRating",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogRequest {
    pub media: MediaKind,
    pub kind: CatalogKind,
    pub skip: usize,
    pub search: Option<String>,
}

impl CatalogRequest {
    pub fn shelf(media: MediaKind, kind: CatalogKind, skip: usize) -> Self {
        CatalogRequest {
            media,
            kind,
            skip,
            search: None,
        }
    }

    pub fn search(media: MediaKind, query: &str) -> Self {
        CatalogRequest {
            media,
            // Only the `top` catalogue accepts a search extra.
            kind: CatalogKind::Popular,
            skip: 0,
            search: Some(query.to_string()),
        }
    }

    fn url(&self) -> String {
        let media = self.media.slug();
        let kind = self.kind.slug();

        match &self.search {
            Some(query) => format!(
                "{CINEMETA}/catalog/{media}/{kind}/search={}.json",
                urlencode(query)
            ),
            // `skip` is a free offset, not a page index — skip=95 is honoured
            // exactly — so a client's window is asked for directly.
            None => match self.skip {
                0 => format!("{CINEMETA}/catalog/{media}/{kind}.json"),
                skip => format!("{CINEMETA}/catalog/{media}/{kind}/skip={skip}.json"),
            },
        }
    }

    /// Never keyed by install token: catalogues are identical for every user,
    /// and a per-token key would multiply the cache by the user count.
    fn cache_key(&self) -> String {
        match &self.search {
            Some(query) => format!(
                "atlas:jellyfin:search:{}:{}",
                self.media.slug(),
                query.to_lowercase()
            ),
            None => format!(
                "atlas:jellyfin:catalog:{}:{}:{}",
                self.media.slug(),
                self.kind.slug(),
                self.skip
            ),
        }
    }
}

fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            b' ' => "%20".to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// One browsable title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub imdb_id: String,
    pub name: String,
    pub media_type: String,
    #[serde(default)]
    pub year: Option<u32>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub community_rating: Option<f32>,
    #[serde(default)]
    pub runtime_minutes: Option<u32>,
}

impl CatalogEntry {
    pub fn is_series(&self) -> bool {
        self.media_type == "series"
    }
}

/// A series with the episode list Cinemeta publishes alongside it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesMeta {
    pub entry: CatalogEntry,
    pub videos: Vec<EpisodeMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeMeta {
    pub season: u32,
    pub episode: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub released: Option<String>,
    #[serde(default)]
    pub runtime_minutes: Option<u32>,
}

pub fn poster_url(imdb_id: &str) -> String {
    format!("{METAHUB}/poster/medium/{imdb_id}/img")
}

pub fn backdrop_url(imdb_id: &str) -> String {
    format!("{METAHUB}/background/medium/{imdb_id}/img")
}

pub fn logo_url(imdb_id: &str) -> String {
    format!("{METAHUB}/logo/medium/{imdb_id}/img")
}

/// Fetches artwork, returning its content type and bytes.
///
/// Uses the catalogue's own client, so a slow image host cannot consume
/// connections the playback path needs. Posters are small enough to hold whole
/// rather than stream, and a size cap keeps that true even if the host misbehaves.
pub async fn image_bytes(url: &str) -> Option<(String, Vec<u8>)> {
    const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;

    let response = CATALOG_CLIENT.get(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    let bytes = response.bytes().await.ok()?;
    if bytes.len() > MAX_IMAGE_BYTES {
        tracing::warn!(url, size = bytes.len(), "artwork larger than expected");
        return None;
    }

    Some((content_type, bytes.to_vec()))
}

// ---------------------------------------------------------------------------
// Cinemeta wire types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CatalogResponse {
    #[serde(default)]
    metas: Vec<CinemetaEntry>,
}

#[derive(Deserialize)]
struct MetaResponse {
    meta: Option<CinemetaDetail>,
}

#[derive(Deserialize)]
struct CinemetaEntry {
    id: Option<String>,
    name: Option<String>,
    #[serde(rename = "type")]
    media_type: Option<String>,
    #[serde(rename = "releaseInfo")]
    release_info: Option<String>,
    description: Option<String>,
    #[serde(default)]
    genres: Vec<String>,
    #[serde(rename = "imdbRating")]
    imdb_rating: Option<String>,
    runtime: Option<String>,
}

#[derive(Deserialize)]
struct CinemetaDetail {
    #[serde(flatten)]
    entry: CinemetaEntry,
    #[serde(default)]
    videos: Vec<CinemetaVideoDetail>,
}

#[derive(Deserialize)]
struct CinemetaVideoDetail {
    season: Option<u32>,
    episode: Option<u32>,
    /// Cinemeta has used both spellings over time.
    #[serde(alias = "title")]
    name: Option<String>,
    overview: Option<String>,
    released: Option<String>,
    runtime: Option<String>,
}

impl CinemetaEntry {
    fn into_entry(self) -> Option<CatalogEntry> {
        let imdb_id = self.id?;
        if !imdb_id.starts_with("tt") {
            return None;
        }

        Some(CatalogEntry {
            name: self.name.unwrap_or_else(|| imdb_id.clone()),
            media_type: self.media_type.unwrap_or_else(|| "movie".to_string()),
            year: self.release_info.as_deref().and_then(parse_year),
            description: self.description,
            genres: self.genres,
            community_rating: self.imdb_rating.as_deref().and_then(|r| r.parse().ok()),
            runtime_minutes: self.runtime.as_deref().and_then(parse_runtime),
            imdb_id,
        })
    }
}

fn parse_year(value: &str) -> Option<u32> {
    value
        .split(|c: char| !c.is_ascii_digit())
        .find(|part| part.len() == 4)
        .and_then(|part| part.parse().ok())
}

fn parse_runtime(value: &str) -> Option<u32> {
    let digits: String = value.chars().filter(char::is_ascii_digit).collect();
    digits.parse().ok()
}

// ---------------------------------------------------------------------------
// Caching
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct CachedPage<T> {
    fetched_at: u64,
    payload: T,
}

impl<T> CachedPage<T> {
    fn age(&self) -> Duration {
        Duration::from_secs(now_epoch().saturating_sub(self.fetched_at))
    }
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

/// A small in-process layer behind Redis.
///
/// `engines::cache` is deliberately not reused — its one global mutex sits on
/// the Stremio resolve path — but without a local layer, an unset or
/// unreachable `UPSTASH_REDIS_URL` would leave catalogue browsing with no cache
/// whatsoever and send every screen to Cinemeta. Contention here is between
/// catalogue requests only.
static LOCAL_CACHE: Lazy<std::sync::RwLock<std::collections::HashMap<String, (u64, String)>>> =
    Lazy::new(|| std::sync::RwLock::new(std::collections::HashMap::new()));

/// Catalogue keys are few — a handful of shelves plus one entry per title
/// browsed — but a long-running process browsing endlessly still needs a bound.
const LOCAL_CACHE_CAPACITY: usize = 2_000;

fn read_local(key: &str) -> Option<(u64, String)> {
    LOCAL_CACHE.read().ok()?.get(key).cloned()
}

fn write_local(key: &str, fetched_at: u64, json: String) {
    let Ok(mut cache) = LOCAL_CACHE.write() else {
        return;
    };
    if cache.len() >= LOCAL_CACHE_CAPACITY && !cache.contains_key(key) {
        // Nothing clever: drop everything and refill. Catalogue entries are
        // cheap to re-fetch and this only happens after thousands of titles.
        cache.clear();
    }
    cache.insert(key.to_string(), (fetched_at, json));
}

async fn read_cache<T: for<'de> Deserialize<'de>>(key: &str) -> Option<CachedPage<T>> {
    if let Some(mut redis_client) = crate::engines::redis::get_redis() {
        let cached: Result<String, _> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut redis_client)
            .await;

        if let Ok(json) = cached {
            if let Ok(page) = serde_json::from_str(&json) {
                return Some(page);
            }
        }
    }

    let (fetched_at, json) = read_local(key)?;
    Some(CachedPage {
        fetched_at,
        payload: serde_json::from_str(&json).ok()?,
    })
}

async fn write_cache<T: Serialize>(key: &str, payload: &T) {
    let fetched_at = now_epoch();
    let entry = CachedPage {
        fetched_at,
        payload,
    };
    let Ok(json) = serde_json::to_string(&entry) else {
        return;
    };

    if let Ok(payload_json) = serde_json::to_string(payload) {
        write_local(key, fetched_at, payload_json);
    }

    let Some(mut redis_client) = crate::engines::redis::get_redis() else {
        return;
    };

    let stored: Result<(), redis::RedisError> = redis::cmd("SETEX")
        .arg(key)
        .arg(HARD_TTL.as_secs())
        .arg(json)
        .query_async(&mut redis_client)
        .await;

    if let Err(error) = stored {
        tracing::warn!(%error, key, "failed to cache catalog page");
    }
}

// ---------------------------------------------------------------------------
// Fetching
// ---------------------------------------------------------------------------

async fn get_json<T: for<'de> Deserialize<'de>>(url: &str) -> Option<T> {
    let response = CATALOG_CLIENT.get(url).send().await.ok()?;
    if !response.status().is_success() {
        tracing::warn!(url, status = %response.status(), "catalog upstream returned an error");
        return None;
    }
    response.json::<T>().await.ok()
}

async fn fetch_catalog(request: &CatalogRequest) -> Vec<CatalogEntry> {
    let url = request.url();
    let Some(response) = get_json::<CatalogResponse>(&url).await else {
        return Vec::new();
    };

    response
        .metas
        .into_iter()
        .filter_map(CinemetaEntry::into_entry)
        .collect()
}

/// Fetches a catalogue page, preferring a cached answer and refreshing a stale
/// one behind the request rather than making the client wait for it.
pub async fn catalog_page(request: CatalogRequest) -> Vec<CatalogEntry> {
    let key = request.cache_key();

    if let Some(cached) = read_cache::<Vec<CatalogEntry>>(&key).await {
        if cached.age() < SOFT_TTL {
            return cached.payload;
        }

        let background = request.clone();
        let background_key = key.clone();
        tokio::spawn(async move {
            let fresh = fetch_catalog(&background).await;
            if !fresh.is_empty() {
                write_cache(&background_key, &fresh).await;
            }
        });

        return cached.payload;
    }

    let entries = fetch_catalog(&request).await;
    // Never cache an empty answer: an upstream blip would otherwise pin an
    // empty shelf in front of every user for hours.
    if !entries.is_empty() {
        write_cache(&key, &entries).await;
    }
    entries
}

/// A window of a catalogue, starting exactly where the client asked.
///
/// Cinemeta honours any `skip`, so the offset needs no alignment; it only takes
/// more than one round trip when a client wants more than a page at once.
pub async fn catalog_slice(
    media: MediaKind,
    kind: CatalogKind,
    start: usize,
    limit: usize,
) -> Vec<CatalogEntry> {
    let mut collected: Vec<CatalogEntry> = Vec::new();
    let mut skip = start;

    while collected.len() < limit {
        let page = catalog_page(CatalogRequest::shelf(media, kind, skip)).await;
        let fetched = page.len();
        collected.extend(page);

        // A short page means upstream has run out; asking again only costs a
        // round trip and returns nothing.
        if fetched < CINEMETA_PAGE {
            break;
        }
        skip += fetched;
    }

    collected.truncate(limit);
    collected
}

pub async fn search(media: MediaKind, query: &str, limit: usize) -> Vec<CatalogEntry> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    catalog_page(CatalogRequest::search(media, query.trim()))
        .await
        .into_iter()
        .take(limit)
        .collect()
}

/// Cinemeta only — deliberately not `get_metadata`, which would also search
/// Torrentio for streams nobody has asked to play yet.
pub async fn series_meta(imdb_id: &str) -> Option<SeriesMeta> {
    let key = format!("atlas:jellyfin:series:{imdb_id}");

    if let Some(cached) = read_cache::<SeriesMeta>(&key).await {
        if cached.age() < HARD_TTL {
            return Some(cached.payload);
        }
    }

    let url = format!("{CINEMETA}/meta/series/{imdb_id}.json");
    let detail = get_json::<MetaResponse>(&url).await?.meta?;

    let videos = detail
        .videos
        .iter()
        .filter_map(|video| {
            Some(EpisodeMeta {
                season: video.season?,
                episode: video.episode?,
                name: video.name.clone(),
                overview: video.overview.clone(),
                released: video.released.clone(),
                runtime_minutes: video.runtime.as_deref().and_then(parse_runtime),
            })
        })
        .collect();

    let meta = SeriesMeta {
        entry: detail.entry.into_entry()?,
        videos,
    };

    write_cache(&key, &meta).await;
    Some(meta)
}

/// A single title's details, for an item page.
pub async fn title_meta(imdb_id: &str, media: MediaKind) -> Option<CatalogEntry> {
    if media == MediaKind::Series {
        return series_meta(imdb_id).await.map(|meta| meta.entry);
    }

    let key = format!("atlas:jellyfin:movie:{imdb_id}");
    if let Some(cached) = read_cache::<CatalogEntry>(&key).await {
        if cached.age() < HARD_TTL {
            return Some(cached.payload);
        }
    }

    let url = format!("{CINEMETA}/meta/movie/{imdb_id}.json");
    let entry = get_json::<MetaResponse>(&url)
        .await?
        .meta?
        .entry
        .into_entry()?;

    write_cache(&key, &entry).await;
    Some(entry)
}

#[cfg(test)]
mod tests {
    use super::{
        backdrop_url, parse_runtime, parse_year, poster_url, urlencode, CatalogKind,
        CatalogRequest, MediaKind,
    };

    #[test]
    fn shelf_urls_match_cinemetas_catalog_routes() {
        assert_eq!(
            CatalogRequest::shelf(MediaKind::Movie, CatalogKind::Popular, 0).url(),
            "https://v3-cinemeta.strem.io/catalog/movie/top.json"
        );
        assert_eq!(
            CatalogRequest::shelf(MediaKind::Series, CatalogKind::New, 100).url(),
            "https://v3-cinemeta.strem.io/catalog/series/year/skip=100.json"
        );
        assert_eq!(
            CatalogRequest::shelf(MediaKind::Movie, CatalogKind::Featured, 0).url(),
            "https://v3-cinemeta.strem.io/catalog/movie/imdbRating.json"
        );
    }

    #[test]
    fn an_arbitrary_offset_is_asked_for_directly() {
        // Verified against Cinemeta: skip is a free offset, not a page index, so
        // a client's window needs no alignment and loses no items to rounding.
        let request = CatalogRequest::shelf(MediaKind::Movie, CatalogKind::Popular, 95);

        assert!(request.url().ends_with("skip=95.json"));
        assert!(request.cache_key().ends_with(":95"));
    }

    #[test]
    fn search_only_uses_the_catalog_that_supports_it() {
        // Cinemeta accepts a search extra on `top` alone.
        let request = CatalogRequest::search(MediaKind::Series, "breaking bad");

        assert_eq!(request.kind, CatalogKind::Popular);
        assert_eq!(
            request.url(),
            "https://v3-cinemeta.strem.io/catalog/series/top/search=breaking%20bad.json"
        );
    }

    #[test]
    fn cache_keys_never_include_a_token_and_separate_every_shelf() {
        let popular = CatalogRequest::shelf(MediaKind::Movie, CatalogKind::Popular, 0);
        let new = CatalogRequest::shelf(MediaKind::Movie, CatalogKind::New, 0);
        let series = CatalogRequest::shelf(MediaKind::Series, CatalogKind::Popular, 0);

        assert_ne!(popular.cache_key(), new.cache_key());
        assert_ne!(popular.cache_key(), series.cache_key());
        assert_eq!(
            popular.cache_key(),
            "atlas:jellyfin:catalog:movie:top:0",
            "catalogues are identical for every user, so the key must not vary by one"
        );
    }

    #[test]
    fn searches_share_a_cache_entry_regardless_of_casing() {
        assert_eq!(
            CatalogRequest::search(MediaKind::Movie, "The Matrix").cache_key(),
            CatalogRequest::search(MediaKind::Movie, "the matrix").cache_key()
        );
    }

    #[test]
    fn queries_are_escaped_for_a_path_segment() {
        assert_eq!(urlencode("breaking bad"), "breaking%20bad");
        assert_eq!(urlencode("rock&roll"), "rock%26roll");
        assert_eq!(urlencode("a/b"), "a%2Fb");
        assert_eq!(urlencode("plain-title_1.0~"), "plain-title_1.0~");
    }

    #[test]
    fn artwork_urls_are_derived_from_the_imdb_id_alone() {
        // No catalogue lookup, and no image bytes through Atlas.
        assert_eq!(
            poster_url("tt0133093"),
            "https://images.metahub.space/poster/medium/tt0133093/img"
        );
        assert_eq!(
            backdrop_url("tt0133093"),
            "https://images.metahub.space/background/medium/tt0133093/img"
        );
    }

    #[test]
    fn every_shelf_offset_gets_its_own_cache_entry() {
        // With skip free-form, offsets no longer collapse onto page starts, so
        // neighbouring windows must not share a cached answer.
        let at_zero = CatalogRequest::shelf(MediaKind::Movie, CatalogKind::Popular, 0);
        let at_fifty = CatalogRequest::shelf(MediaKind::Movie, CatalogKind::Popular, 50);

        assert_ne!(at_zero.cache_key(), at_fifty.cache_key());
        assert_eq!(
            at_zero.url(),
            "https://v3-cinemeta.strem.io/catalog/movie/top.json"
        );
    }

    #[test]
    fn parses_cinemeta_release_and_runtime_strings() {
        assert_eq!(parse_year("2011-2019"), Some(2011));
        assert_eq!(parse_year("1999"), Some(1999));
        assert_eq!(parse_year(""), None);
        assert_eq!(parse_runtime("57 min"), Some(57));
        assert_eq!(parse_runtime(""), None);
    }
}
