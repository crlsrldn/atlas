//! Server identity and the startup endpoints clients probe before login.
//!
//! `/System/Info/Public` is the important one: it is unauthenticated, and it is
//! how a client decides whether the URL it was given is a media server at all. A
//! 404 here means the user never reaches a login form.

use crate::api::jellyfin::auth::AuthContext;
use crate::api::jellyfin::dto::{
    BrandingOptions, DisplayPreferencesDto, EndpointInfo, PublicSystemInfo, QueryResult,
    SystemInfo, UserDto, JELLYFIN_VERSION, PRODUCT_NAME,
};
use crate::api::jellyfin::server_id;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router {
    Router::new()
        .route("/System/Info/Public", get(public_system_info))
        .route("/System/Info", get(system_info))
        .route("/System/Endpoint", get(endpoint_info))
        .route("/Branding/Configuration", get(branding))
        .route("/Branding/Css", get(branding_css))
        .route("/QuickConnect/Enabled", get(quick_connect_enabled))
        .route("/Users/Public", get(public_users))
        // Any id, not just "usersettings": clients ask under several.
        .route(
            "/DisplayPreferences/:preferences_id",
            get(display_preferences).post(save_display_preferences),
        )
        .route("/Library/VirtualFolders", get(empty_list))
        .route("/Library/MediaFolders", get(empty_query_result))
        // Startup probes. None of these carry anything Atlas has, but a 404
        // stops a client mid-flow, so each answers with a well-formed nothing.
        // Shape matters: a client expecting an array and given an object fails
        // the same way it would on a 404.
        .route("/UserViews/GroupingOptions", get(empty_list))
        .route("/Users/:user_id/GroupingOptions", get(empty_list))
        .route("/Localization/Options", get(empty_list))
        .route("/Localization/Cultures", get(empty_list))
        .route("/Localization/Countries", get(empty_list))
        .route("/Plugins", get(empty_list))
        .route("/ScheduledTasks", get(empty_list))
        .route("/Devices", get(empty_query_result))
        .route("/Channels", get(empty_query_result))
        .route("/Genres", get(empty_query_result))
        .route("/Studios", get(empty_query_result))
        .route("/Persons", get(empty_query_result))
        .route("/Playlists", get(empty_query_result))
        .route("/Shows/Upcoming", get(empty_query_result))
        .route("/Items/Filters", get(item_filters))
        .route("/Items/Filters2", get(item_filters))
        .route("/LiveTv/Info", get(live_tv_info))
}

pub fn public_base_url() -> String {
    std::env::var("ATLAS_PUBLIC_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
}

async fn public_system_info() -> Json<PublicSystemInfo> {
    Json(PublicSystemInfo {
        local_address: public_base_url(),
        server_name: PRODUCT_NAME.to_string(),
        version: JELLYFIN_VERSION.to_string(),
        product_name: PRODUCT_NAME.to_string(),
        operating_system: String::new(),
        id: server_id(),
        startup_wizard_completed: true,
    })
}

async fn system_info(_auth: AuthContext) -> Json<SystemInfo> {
    Json(SystemInfo {
        local_address: public_base_url(),
        server_name: PRODUCT_NAME.to_string(),
        version: JELLYFIN_VERSION.to_string(),
        product_name: PRODUCT_NAME.to_string(),
        operating_system: String::new(),
        id: server_id(),
        startup_wizard_completed: true,
        has_pending_restart: false,
        is_shutting_down: false,
        supports_library_monitor: false,
        has_update_available: false,
        can_launch_web_browser: false,
        transcoding_temp_path: None,
        cache_path: None,
        package_name: None,
    })
}

async fn endpoint_info() -> Json<EndpointInfo> {
    Json(EndpointInfo {
        is_local: false,
        is_in_network: false,
    })
}

async fn branding() -> Json<BrandingOptions> {
    Json(BrandingOptions {
        login_disclaimer: None,
        custom_css: None,
        splashscreen_enabled: false,
    })
}

async fn branding_css() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css")],
        String::new(),
    )
}

async fn quick_connect_enabled() -> Json<bool> {
    Json(false)
}

/// Deliberately empty.
///
/// A 404 stops some clients rendering the login form at all, while a *populated*
/// list makes them show a user picker. An empty list is what asks for a username
/// and password, which is the flow Atlas needs — the password is the install
/// token.
async fn public_users() -> Json<Vec<UserDto>> {
    Json(Vec::new())
}

#[derive(Debug, Deserialize)]
struct DisplayPreferencesQuery {
    #[serde(default)]
    client: Option<String>,
}

/// Must be well-formed rather than `{}`: an empty object here is a long-standing
/// cause of Jellyfin client crashes at startup.
async fn display_preferences(
    Path(preferences_id): Path<String>,
    Query(query): Query<DisplayPreferencesQuery>,
) -> Json<DisplayPreferencesDto> {
    Json(DisplayPreferencesDto::defaults(
        preferences_id,
        query.client.unwrap_or_else(|| "emby".to_string()),
    ))
}

async fn save_display_preferences() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn empty_list() -> Json<Vec<serde_json::Value>> {
    Json(Vec::new())
}

async fn empty_query_result() -> Json<QueryResult<serde_json::Value>> {
    Json(QueryResult::empty())
}

/// Atlas has no facets to filter by, but the keys have to exist: clients index
/// into this rather than checking for it.
async fn item_filters() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "Genres": [],
        "Tags": [],
        "OfficialRatings": [],
        "Years": []
    }))
}

async fn live_tv_info() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "Services": [],
        "IsEnabled": false,
        "EnabledUsers": []
    }))
}

#[cfg(test)]
mod tests {
    use super::{public_system_info, public_users, quick_connect_enabled};

    #[tokio::test]
    async fn public_system_info_identifies_the_server_without_auth() {
        let info = public_system_info().await.0;

        assert_eq!(info.product_name, "Atlas");
        assert!(info.startup_wizard_completed);
        assert_eq!(info.id.len(), 32);
    }

    #[tokio::test]
    async fn public_users_is_empty_so_clients_ask_for_a_password() {
        assert!(public_users().await.0.is_empty());
    }

    #[tokio::test]
    async fn quick_connect_is_off() {
        assert!(!quick_connect_enabled().await.0);
    }
}
