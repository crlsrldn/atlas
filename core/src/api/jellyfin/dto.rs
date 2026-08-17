//! Jellyfin wire types.
//!
//! Field names are Jellyfin's, so everything is `PascalCase`. Note the absence
//! of `skip_serializing_if`: real Jellyfin emits explicit nulls, and several
//! clients decode into types that require the key to be present, so an omitted
//! field is not the same as a null one here.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Advertised to clients. Atlas is not Jellyfin, but a client that does not
/// recognise the version may refuse to talk to it at all.
pub const JELLYFIN_VERSION: &str = "10.10.3";
pub const PRODUCT_NAME: &str = "Atlas";

pub fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// The unauthenticated probe a client uses to decide whether a URL is a
/// Jellyfin server at all.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublicSystemInfo {
    pub local_address: String,
    pub server_name: String,
    pub version: String,
    pub product_name: String,
    pub operating_system: String,
    pub id: String,
    pub startup_wizard_completed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SystemInfo {
    pub local_address: String,
    pub server_name: String,
    pub version: String,
    pub product_name: String,
    pub operating_system: String,
    pub id: String,
    pub startup_wizard_completed: bool,
    pub has_pending_restart: bool,
    pub is_shutting_down: bool,
    pub supports_library_monitor: bool,
    pub has_update_available: bool,
    pub can_launch_web_browser: bool,
    pub transcoding_temp_path: Option<String>,
    pub cache_path: Option<String>,
    pub package_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EndpointInfo {
    pub is_local: bool,
    pub is_in_network: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BrandingOptions {
    pub login_disclaimer: Option<String>,
    pub custom_css: Option<String>,
    pub splashscreen_enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserPolicy {
    pub is_administrator: bool,
    pub is_hidden: bool,
    pub is_disabled: bool,
    pub enable_media_playback: bool,
    pub enable_audio_playback_transcoding: bool,
    pub enable_video_playback_transcoding: bool,
    pub enable_playback_remuxing: bool,
    pub enable_content_downloading: bool,
    pub enable_sync_transcoding: bool,
    pub enable_all_devices: bool,
    pub enable_all_folders: bool,
    pub enable_all_channels: bool,
    pub enable_remote_access: bool,
    pub enabled_folders: Vec<String>,
    pub blocked_tags: Vec<String>,
    pub access_schedules: Vec<serde_json::Value>,
    pub remote_client_bitrate_limit: i64,
}

impl Default for UserPolicy {
    fn default() -> Self {
        UserPolicy {
            is_administrator: false,
            is_hidden: false,
            is_disabled: false,
            enable_media_playback: true,
            // Atlas never transcodes: it hands back a redirect to a CDN. Saying
            // otherwise invites a client to ask for a transcode that cannot be
            // produced.
            enable_audio_playback_transcoding: false,
            enable_video_playback_transcoding: false,
            enable_playback_remuxing: false,
            enable_content_downloading: true,
            enable_sync_transcoding: false,
            enable_all_devices: true,
            enable_all_folders: true,
            enable_all_channels: false,
            enable_remote_access: true,
            enabled_folders: Vec::new(),
            blocked_tags: Vec::new(),
            access_schedules: Vec::new(),
            remote_client_bitrate_limit: 0,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserConfiguration {
    pub play_default_audio_track: bool,
    pub subtitle_language_preference: String,
    pub display_missing_episodes: bool,
    pub grouped_folders: Vec<String>,
    pub subtitle_mode: String,
    pub display_collections_view: bool,
    pub enable_local_password: bool,
    pub ordered_views: Vec<String>,
    pub latest_items_excludes: Vec<String>,
    pub my_media_excludes: Vec<String>,
    pub hide_played_in_latest: bool,
    pub remember_audio_selections: bool,
    pub remember_subtitle_selections: bool,
    pub enable_next_episode_auto_play: bool,
}

impl Default for UserConfiguration {
    fn default() -> Self {
        UserConfiguration {
            play_default_audio_track: true,
            subtitle_language_preference: String::new(),
            display_missing_episodes: false,
            grouped_folders: Vec::new(),
            subtitle_mode: "Default".to_string(),
            display_collections_view: false,
            enable_local_password: false,
            ordered_views: Vec::new(),
            latest_items_excludes: Vec::new(),
            my_media_excludes: Vec::new(),
            hide_played_in_latest: true,
            remember_audio_selections: true,
            remember_subtitle_selections: true,
            enable_next_episode_auto_play: true,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserDto {
    pub name: String,
    pub server_id: String,
    pub id: String,
    pub has_password: bool,
    pub has_configured_password: bool,
    pub has_configured_easy_password: bool,
    pub enable_auto_login: bool,
    pub last_login_date: Option<String>,
    pub last_activity_date: Option<String>,
    pub configuration: UserConfiguration,
    pub policy: UserPolicy,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionInfoDto {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub client: String,
    pub device_name: String,
    pub device_id: String,
    pub application_version: String,
    pub server_id: String,
    pub supports_remote_control: bool,
    pub is_active: bool,
    pub has_custom_device_name: bool,
    pub now_playing_queue: Vec<serde_json::Value>,
    pub playable_media_types: Vec<String>,
    pub supported_commands: Vec<String>,
    pub last_activity_date: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthenticationResult {
    pub user: UserDto,
    pub session_info: SessionInfoDto,
    pub access_token: String,
    pub server_id: String,
}

/// A request body read case-insensitively, tolerating the same value arriving
/// under more than one spelling.
///
/// Serde aliases cannot express this. Two keys that map to one field are a
/// *duplicate field* error, so a client sending both `Pw` and `Password` —
/// which Infuse does — has its entire login body rejected with a 422 before any
/// handler sees it. Reading the object directly accepts whatever arrives, which
/// is the same reasoning [`super::query`] applies to query parameters.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
pub struct JellyfinBody(serde_json::Value);

impl JellyfinBody {
    pub fn new(value: serde_json::Value) -> Self {
        JellyfinBody(value)
    }

    pub fn into_value(self) -> serde_json::Value {
        self.0
    }

    /// The first of `names` present, compared without regard to case.
    pub fn get(&self, names: &[&str]) -> Option<&serde_json::Value> {
        let object = self.0.as_object()?;

        names.iter().find_map(|wanted| {
            object.iter().find_map(|(key, value)| {
                (key.eq_ignore_ascii_case(wanted) && !value.is_null()).then_some(value)
            })
        })
    }

    pub fn string(&self, names: &[&str]) -> Option<String> {
        self.get(names)?.as_str().map(str::to_string)
    }

    pub fn integer(&self, names: &[&str]) -> Option<i64> {
        let value = self.get(names)?;
        value
            .as_i64()
            .or_else(|| value.as_f64().map(|number| number as i64))
    }

    /// A nested object, still read case-insensitively.
    pub fn object(&self, names: &[&str]) -> Option<JellyfinBody> {
        Some(JellyfinBody(self.get(names)?.clone()))
    }

    pub fn array(&self, names: &[&str]) -> Vec<JellyfinBody> {
        self.get(names)
            .and_then(serde_json::Value::as_array)
            .map(|items| items.iter().cloned().map(JellyfinBody).collect())
            .unwrap_or_default()
    }
}

/// Infuse requires a username field when adding a server even though Atlas
/// authenticates on the install token alone, so the username is read and
/// ignored.
impl JellyfinBody {
    pub fn username(&self) -> Option<String> {
        self.string(&["Username", "User", "Name"])
    }

    pub fn password(&self) -> Option<String> {
        self.string(&["Pw", "Password"])
    }
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserItemDataDto {
    pub playback_position_ticks: i64,
    pub play_count: i32,
    pub is_favorite: bool,
    pub played: bool,
    pub played_percentage: Option<f64>,
    pub key: String,
}

/// The subset of Jellyfin's `BaseItemDto` Atlas can populate honestly. Grows as
/// later phases add real catalogue data.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BaseItemDto {
    pub name: String,
    pub server_id: String,
    pub id: String,
    pub etag: Option<String>,
    pub date_created: Option<String>,
    pub can_delete: bool,
    pub can_download: bool,
    pub sort_name: Option<String>,
    pub premiere_date: Option<String>,
    pub external_urls: Vec<serde_json::Value>,
    pub path: Option<String>,
    pub overview: Option<String>,
    pub taglines: Vec<String>,
    pub genres: Vec<String>,
    pub community_rating: Option<f32>,
    pub run_time_ticks: Option<i64>,
    pub production_year: Option<i32>,
    pub index_number: Option<i32>,
    pub parent_index_number: Option<i32>,
    pub is_folder: bool,
    pub parent_id: Option<String>,
    #[serde(rename = "Type")]
    pub item_type: String,
    pub studios: Vec<serde_json::Value>,
    pub genre_items: Vec<serde_json::Value>,
    pub series_name: Option<String>,
    pub series_id: Option<String>,
    pub season_id: Option<String>,
    pub season_name: Option<String>,
    pub user_data: Option<UserItemDataDto>,
    pub child_count: Option<i32>,
    pub display_preferences_id: Option<String>,
    pub tags: Vec<String>,
    pub primary_image_aspect_ratio: Option<f64>,
    pub collection_type: Option<String>,
    pub image_tags: HashMap<String, String>,
    pub backdrop_image_tags: Vec<String>,
    pub location_type: String,
    pub media_type: String,
    pub provider_ids: HashMap<String, String>,
    /// Always empty while browsing. Sources are resolved only by
    /// `PlaybackInfo`; filling this during enumeration would fire a provider
    /// search for every tile on screen.
    pub media_sources: Vec<serde_json::Value>,
    pub media_streams: Vec<serde_json::Value>,
}

impl BaseItemDto {
    /// A folder-shaped item with every collection field empty rather than
    /// absent, ready for a caller to fill in the parts it knows.
    pub fn folder(id: String, name: String, server_id: String) -> Self {
        BaseItemDto {
            sort_name: Some(name.to_lowercase()),
            name,
            server_id,
            id,
            etag: None,
            date_created: None,
            can_delete: false,
            can_download: false,
            premiere_date: None,
            external_urls: Vec::new(),
            path: None,
            overview: None,
            taglines: Vec::new(),
            genres: Vec::new(),
            community_rating: None,
            run_time_ticks: None,
            production_year: None,
            index_number: None,
            parent_index_number: None,
            is_folder: true,
            parent_id: None,
            item_type: "Folder".to_string(),
            studios: Vec::new(),
            genre_items: Vec::new(),
            series_name: None,
            series_id: None,
            season_id: None,
            season_name: None,
            user_data: Some(UserItemDataDto::default()),
            child_count: None,
            display_preferences_id: None,
            tags: Vec::new(),
            primary_image_aspect_ratio: None,
            collection_type: None,
            image_tags: HashMap::new(),
            backdrop_image_tags: Vec::new(),
            location_type: "Virtual".to_string(),
            media_type: "Unknown".to_string(),
            provider_ids: HashMap::new(),
            media_sources: Vec::new(),
            media_streams: Vec::new(),
        }
    }
}

/// Jellyfin's paged envelope. `TotalRecordCount` drives client scrollbars and,
/// crucially, tells a client when to stop paging.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueryResult<T> {
    pub items: Vec<T>,
    pub total_record_count: i32,
    pub start_index: i32,
}

impl<T> QueryResult<T> {
    pub fn new(items: Vec<T>, total: i32, start_index: i32) -> Self {
        QueryResult {
            items,
            total_record_count: total,
            start_index,
        }
    }

    /// A whole, unpaged result. Also the shape every handler returns instead of
    /// an error: clients degrade gracefully on an empty list and badly on a 500.
    pub fn complete(items: Vec<T>) -> Self {
        let total = items.len() as i32;
        QueryResult::new(items, total, 0)
    }

    pub fn empty() -> Self {
        QueryResult::new(Vec::new(), 0, 0)
    }
}

/// Must be well-formed rather than `{}` — a malformed or missing response here
/// is a long-standing cause of client crashes at startup.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DisplayPreferencesDto {
    pub id: String,
    pub view_type: Option<String>,
    pub sort_by: String,
    pub index_by: Option<String>,
    pub remember_indexing: bool,
    pub primary_image_height: i32,
    pub primary_image_width: i32,
    pub custom_prefs: HashMap<String, String>,
    pub scroll_direction: String,
    pub show_backdrop: bool,
    pub remember_sorting: bool,
    pub sort_order: String,
    pub show_sidebar: bool,
    pub client: String,
}

impl DisplayPreferencesDto {
    pub fn defaults(id: String, client: String) -> Self {
        DisplayPreferencesDto {
            id,
            view_type: None,
            sort_by: "SortName".to_string(),
            index_by: None,
            remember_indexing: false,
            primary_image_height: 250,
            primary_image_width: 250,
            custom_prefs: HashMap::new(),
            scroll_direction: "Horizontal".to_string(),
            show_backdrop: true,
            remember_sorting: false,
            sort_order: "Ascending".to_string(),
            show_sidebar: false,
            client,
        }
    }
}

/// One track inside a source. Atlas never opens the file, so these are
/// synthesised from the release name and are advisory: the client parses the
/// real container once playback starts. They decide which version a viewer
/// picks, not whether it plays.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MediaStream {
    #[serde(rename = "Type")]
    pub stream_type: String,
    pub index: i32,
    pub codec: Option<String>,
    pub language: Option<String>,
    pub title: Option<String>,
    pub display_title: Option<String>,
    pub is_default: bool,
    pub is_forced: bool,
    pub is_external: bool,
    pub is_interlaced: bool,
    pub bit_rate: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub aspect_ratio: Option<String>,
    pub video_range: Option<String>,
    pub video_range_type: Option<String>,
    pub channels: Option<i32>,
    pub channel_layout: Option<String>,
    /// Left null on purpose. A wrong profile makes a client pre-emptively
    /// reject a source it could have played.
    pub profile: Option<String>,
}

impl MediaStream {
    pub fn video(index: i32) -> Self {
        MediaStream {
            stream_type: "Video".to_string(),
            index,
            codec: None,
            language: None,
            title: None,
            display_title: None,
            is_default: true,
            is_forced: false,
            is_external: false,
            is_interlaced: false,
            bit_rate: None,
            width: None,
            height: None,
            aspect_ratio: Some("16:9".to_string()),
            video_range: None,
            video_range_type: None,
            channels: None,
            channel_layout: None,
            profile: None,
        }
    }

    pub fn audio(index: i32) -> Self {
        MediaStream {
            stream_type: "Audio".to_string(),
            ..MediaStream::video(index)
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MediaSourceInfo {
    /// The infohash. Jellyfin's media source id is a free-form string, not a
    /// GUID, so keeping it whole makes the stream URL self-describing.
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub protocol: String,
    #[serde(rename = "Type")]
    pub source_type: String,
    pub container: Option<String>,
    pub size: Option<i64>,
    pub bitrate: Option<i64>,
    pub run_time_ticks: Option<i64>,
    pub is_remote: bool,
    pub supports_direct_play: bool,
    pub supports_direct_stream: bool,
    /// Always false. Atlas hands back a redirect; a client told transcoding is
    /// available will ask for a transcode URL that cannot be produced.
    pub supports_transcoding: bool,
    pub supports_probing: bool,
    pub requires_opening: bool,
    pub requires_closing: bool,
    pub requires_looping: bool,
    pub is_infinite_stream: bool,
    pub read_at_native_framerate: bool,
    pub ignore_dts: bool,
    pub ignore_index: bool,
    pub gen_pts_input: bool,
    pub video_type: String,
    pub media_streams: Vec<MediaStream>,
    pub media_attachments: Vec<serde_json::Value>,
    pub formats: Vec<String>,
    pub default_audio_stream_index: Option<i32>,
    pub default_subtitle_stream_index: Option<i32>,
    pub transcoding_url: Option<String>,
    pub transcoding_container: Option<String>,
    pub transcoding_sub_protocol: Option<String>,
    pub etag: Option<String>,
}

impl MediaSourceInfo {
    pub fn direct_play(id: String, name: String) -> Self {
        MediaSourceInfo {
            id,
            name,
            path: None,
            protocol: "Http".to_string(),
            source_type: "Default".to_string(),
            container: None,
            size: None,
            bitrate: None,
            run_time_ticks: None,
            is_remote: true,
            supports_direct_play: true,
            supports_direct_stream: true,
            supports_transcoding: false,
            supports_probing: false,
            requires_opening: false,
            requires_closing: false,
            requires_looping: false,
            is_infinite_stream: false,
            read_at_native_framerate: false,
            ignore_dts: false,
            ignore_index: false,
            gen_pts_input: false,
            video_type: "VideoFile".to_string(),
            media_streams: Vec::new(),
            media_attachments: Vec::new(),
            formats: Vec::new(),
            default_audio_stream_index: Some(1),
            default_subtitle_stream_index: None,
            transcoding_url: None,
            transcoding_container: None,
            transcoding_sub_protocol: None,
            etag: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlaybackInfoResponse {
    pub media_sources: Vec<MediaSourceInfo>,
    pub play_session_id: String,
    pub error_code: Option<String>,
}

/// What a client posts to `PlaybackInfo`. The device profile is the part worth
/// having: it states real codec support and a bitrate ceiling, which is far
/// better evidence than guessing from a User-Agent.
impl JellyfinBody {
    pub fn device_profile(&self) -> Option<JellyfinBody> {
        self.object(&["DeviceProfile"])
    }

    /// A ceiling declared on the request itself or inside the profile.
    pub fn max_streaming_bitrate(&self) -> Option<i64> {
        self.integer(&["MaxStreamingBitrate"])
            .or_else(|| self.device_profile()?.integer(&["MaxStreamingBitrate"]))
    }

    /// Whether the client listed a video codec at all, and if so whether this
    /// one is among them. A profile that lists none claims nothing, so nothing
    /// is excluded on its behalf.
    pub fn supports_video_codec(&self, codec: &str) -> Option<bool> {
        let profile = self.device_profile()?;
        let listed: Vec<String> = profile
            .array(&["DirectPlayProfiles"])
            .iter()
            .filter_map(|entry| entry.string(&["VideoCodec"]))
            .flat_map(|codecs| {
                codecs
                    .split(',')
                    .map(|codec| codec.trim().to_ascii_lowercase())
                    .collect::<Vec<_>>()
            })
            .filter(|codec| !codec.is_empty())
            .collect();

        if listed.is_empty() {
            return None;
        }
        Some(
            listed
                .iter()
                .any(|entry| entry == &codec.to_ascii_lowercase()),
        )
    }

    /// Fields a playback report carries.
    pub fn item_id(&self) -> Option<String> {
        self.string(&["ItemId", "Id"])
    }

    pub fn position_ticks(&self) -> Option<i64> {
        self.integer(&["PositionTicks"])
    }

    pub fn run_time_ticks(&self) -> Option<i64> {
        self.integer(&["RunTimeTicks"])
    }
}

#[cfg(test)]
mod tests {
    use super::{BaseItemDto, QueryResult, UserItemDataDto};

    #[test]
    fn items_serialize_with_jellyfin_field_names() {
        let item = BaseItemDto::folder(
            "abc".to_string(),
            "Movies".to_string(),
            "server".to_string(),
        );
        let json = serde_json::to_value(&item).expect("item must serialize");

        assert_eq!(json["Name"], "Movies");
        assert_eq!(json["Id"], "abc");
        assert_eq!(json["IsFolder"], true);
        assert_eq!(json["Type"], "Folder");
    }

    #[test]
    fn absent_values_serialize_as_explicit_nulls() {
        // Clients decode into types that require the key to exist, so a missing
        // key is not interchangeable with a null one.
        let item = BaseItemDto::folder("abc".to_string(), "M".to_string(), "s".to_string());
        let json = serde_json::to_value(&item).expect("item must serialize");

        assert!(json.get("Overview").is_some());
        assert!(json["Overview"].is_null());
        assert!(json.get("RunTimeTicks").is_some());
        assert!(json["RunTimeTicks"].is_null());
    }

    #[test]
    fn browsing_never_advertises_media_sources() {
        let item = BaseItemDto::folder("abc".to_string(), "M".to_string(), "s".to_string());
        let json = serde_json::to_value(&item).expect("item must serialize");

        assert_eq!(json["MediaSources"], serde_json::json!([]));
    }

    #[test]
    fn query_results_report_their_own_length() {
        let result = QueryResult::complete(vec![UserItemDataDto::default(); 3]);
        let json = serde_json::to_value(&result).expect("result must serialize");

        assert_eq!(json["TotalRecordCount"], 3);
        assert_eq!(json["StartIndex"], 0);
        assert_eq!(json["Items"].as_array().map(Vec::len), Some(3));
    }

    #[test]
    fn a_value_arriving_under_two_spellings_does_not_reject_the_body() {
        // The bug this type exists for: Infuse sends both Pw and Password, and
        // serde aliases treat that as a duplicate field, failing the whole body
        // with a 422 before any handler runs.
        let body: super::JellyfinBody =
            serde_json::from_str(r#"{"Username":"atlas","Pw":"secret","Password":"secret"}"#)
                .expect("a body with both spellings must parse");

        assert_eq!(body.password().as_deref(), Some("secret"));
        assert_eq!(body.username().as_deref(), Some("atlas"));
    }

    #[test]
    fn fields_are_found_whatever_casing_a_client_uses() {
        let camel: super::JellyfinBody =
            serde_json::from_str(r#"{"username":"atlas","pw":"secret"}"#).expect("valid");

        assert_eq!(camel.username().as_deref(), Some("atlas"));
        assert_eq!(camel.password().as_deref(), Some("secret"));
    }

    #[test]
    fn nulls_and_missing_fields_read_the_same_way() {
        let explicit: super::JellyfinBody =
            serde_json::from_str(r#"{"Username":null,"Pw":null}"#).expect("valid");
        let absent: super::JellyfinBody = serde_json::from_str("{}").expect("valid");

        assert_eq!(explicit.password(), None);
        assert_eq!(absent.password(), None);
    }

    #[test]
    fn a_device_profile_is_read_through_the_same_tolerance() {
        let body: super::JellyfinBody = serde_json::from_str(
            r#"{"DeviceProfile":{"MaxStreamingBitrate":120000000,
                "DirectPlayProfiles":[{"Container":"mkv","VideoCodec":"h264,hevc"}]}}"#,
        )
        .expect("valid");

        assert_eq!(body.max_streaming_bitrate(), Some(120_000_000));
        assert_eq!(body.supports_video_codec("hevc"), Some(true));
        assert_eq!(body.supports_video_codec("av1"), Some(false));
        // A profile listing nothing claims nothing.
        let bare: super::JellyfinBody =
            serde_json::from_str(r#"{"DeviceProfile":{}}"#).expect("valid");
        assert_eq!(bare.supports_video_codec("hevc"), None);
    }

    #[test]
    fn a_body_that_is_not_an_object_is_simply_empty() {
        // Some clients post an empty string or a bare array; that is not worth
        // an error response.
        let list: super::JellyfinBody = serde_json::from_str("[]").expect("valid");

        assert_eq!(list.item_id(), None);
        assert_eq!(list.position_ticks(), None);
    }

    #[test]
    fn empty_results_are_well_formed() {
        let result: QueryResult<UserItemDataDto> = QueryResult::empty();
        let json = serde_json::to_value(&result).expect("result must serialize");

        assert_eq!(json["Items"], serde_json::json!([]));
        assert_eq!(json["TotalRecordCount"], 0);
    }
}
