//! Login and the library list.

use crate::api::jellyfin::auth::{parse_authorization, AuthContext};
use crate::api::jellyfin::dto::{
    now_iso8601, AuthenticationResult, BaseItemDto, JellyfinBody, QueryResult, SessionInfoDto,
    UserConfiguration, UserDto, UserPolicy,
};
use crate::api::jellyfin::ids::{ItemId, Library};
use crate::api::jellyfin::{server_id, stable_hex_id};
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/Users/AuthenticateByName", post(authenticate_by_name))
        .route("/Users/Me", get(current_user))
        .route("/Users/:user_id", get(user_by_id))
        .route("/Users/:user_id/Views", get(views))
        .route("/Users/:user_id/Items/Root", get(root_item))
        // Jellyfin 10.9 moved these off the user path and onto a userId query.
        // Infuse 8.5 uses the newer shape; older clients use the one above, so
        // both are served.
        .route("/UserViews", get(views_flat))
        .route("/UserItems/Root", get(root_item_flat))
}

fn header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn user_dto(name: String, user_id: String, server: String) -> UserDto {
    UserDto {
        name,
        server_id: server,
        id: user_id,
        has_password: true,
        has_configured_password: true,
        has_configured_easy_password: false,
        enable_auto_login: false,
        last_login_date: Some(now_iso8601()),
        last_activity_date: Some(now_iso8601()),
        configuration: UserConfiguration::default(),
        policy: UserPolicy::default(),
    }
}

/// Infuse requires a username field when adding a server, so any username is
/// accepted; the password is the install token and is the only thing that
/// identifies the caller.
///
/// The token is validated at the gateway, which owns the Supabase lookup. By the
/// time a request reaches core it has already been accepted, so this only has to
/// mint the session Infuse will carry from here on.
async fn authenticate_by_name(headers: HeaderMap, Json(request): Json<JellyfinBody>) -> Response {
    let token = request.password().unwrap_or_default().trim().to_string();

    if token.is_empty() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "Error": "Unauthorized",
                "Message": "Enter your Atlas install token as the password."
            })),
        )
            .into_response();
    }

    let declared = parse_authorization(
        &header(&headers, "x-emby-authorization")
            .or_else(|| header(&headers, "authorization"))
            .unwrap_or_default(),
    );

    let server = server_id();
    let user_id = stable_hex_id("atlas-user", &token);
    let device_id = declared
        .device_id
        .clone()
        .unwrap_or_else(|| "unknown-device".to_string());
    let profile_name =
        header(&headers, "x-atlas-profile-name").unwrap_or_else(|| "Atlas".to_string());

    let session = SessionInfoDto {
        id: stable_hex_id("atlas-session", &format!("{token}:{device_id}")),
        user_id: user_id.clone(),
        user_name: profile_name.clone(),
        client: declared.client.unwrap_or_else(|| "Unknown".to_string()),
        device_name: declared.device.unwrap_or_else(|| "Unknown".to_string()),
        device_id,
        application_version: declared.version.unwrap_or_else(|| "0".to_string()),
        server_id: server.clone(),
        supports_remote_control: false,
        is_active: true,
        has_custom_device_name: false,
        now_playing_queue: Vec::new(),
        playable_media_types: vec!["Video".to_string()],
        supported_commands: Vec::new(),
        last_activity_date: now_iso8601(),
    };

    tracing::info!(
        client = %session.client,
        device = %session.device_name,
        "Jellyfin client authenticated"
    );

    Json(AuthenticationResult {
        user: user_dto(profile_name, user_id, server.clone()),
        session_info: session,
        // Handing the install token straight back keeps the gateway's
        // `loadPreferences` the single place a token is resolved.
        access_token: token,
        server_id: server,
    })
    .into_response()
}

async fn current_user(auth: AuthContext) -> Json<UserDto> {
    Json(user_dto(
        auth.profile_name.clone(),
        auth.user_id(),
        auth.server_id(),
    ))
}

/// The `user_id` in the path is ignored: identity comes from the token, and a
/// client can put anything here.
async fn user_by_id(auth: AuthContext, Path(_user_id): Path<String>) -> Json<UserDto> {
    Json(user_dto(
        auth.profile_name.clone(),
        auth.user_id(),
        auth.server_id(),
    ))
}

fn library_view(library: Library, server: String) -> BaseItemDto {
    let mut item = BaseItemDto::folder(
        ItemId::library(library).to_hex(),
        library.display_name().to_string(),
        server,
    );
    item.item_type = "CollectionFolder".to_string();
    // Clients pick a metadata agent from this; labelling shows as films makes
    // series render wrongly throughout.
    item.collection_type = Some(library.collection_type().to_string());
    item.parent_id = Some(ItemId::root().to_hex());
    item.display_preferences_id = Some("usersettings".to_string());
    item
}

fn user_views(auth: &AuthContext) -> QueryResult<BaseItemDto> {
    let server = auth.server_id();

    QueryResult::complete(vec![
        library_view(Library::Movies, server.clone()),
        library_view(Library::Shows, server),
    ])
}

async fn views(auth: AuthContext, Path(_user_id): Path<String>) -> Json<QueryResult<BaseItemDto>> {
    Json(user_views(&auth))
}

async fn views_flat(auth: AuthContext) -> Json<QueryResult<BaseItemDto>> {
    Json(user_views(&auth))
}

fn root(auth: &AuthContext) -> BaseItemDto {
    let mut item = BaseItemDto::folder(
        ItemId::root().to_hex(),
        "Atlas".to_string(),
        auth.server_id(),
    );
    item.item_type = "Folder".to_string();
    item
}

async fn root_item(auth: AuthContext, Path(_user_id): Path<String>) -> Json<BaseItemDto> {
    Json(root(&auth))
}

async fn root_item_flat(auth: AuthContext) -> Json<BaseItemDto> {
    Json(root(&auth))
}

#[cfg(test)]
mod tests {
    use super::{authenticate_by_name, library_view, views};
    use crate::api::config::UserPreferences;
    use crate::api::jellyfin::auth::AuthContext;
    use crate::api::jellyfin::dto::JellyfinBody;
    use crate::api::jellyfin::ids::{ItemId, Library};
    use axum::http::{HeaderMap, HeaderValue, StatusCode};
    use axum::response::IntoResponse;
    use axum::{extract::Path, Json};

    fn auth_context() -> AuthContext {
        AuthContext {
            token: "token-abc".to_string(),
            prefs: UserPreferences::default(),
            profile_name: "Living Room".to_string(),
            monetization_enabled: false,
            client: Some("Infuse-Direct".to_string()),
            device: Some("Apple TV".to_string()),
            device_id: Some("device-1".to_string()),
            version: Some("8.4".to_string()),
            user_agent: Some("Infuse-Direct/7.7".to_string()),
        }
    }

    #[tokio::test]
    async fn authentication_requires_a_password() {
        let response = authenticate_by_name(
            HeaderMap::new(),
            Json(JellyfinBody::new(serde_json::json!({"Username": "atlas"}))),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn a_password_sent_under_two_names_is_accepted() {
        // Infuse sends both Pw and Password. Serde aliases cannot express that
        // — two keys mapping to one field is a duplicate-field error — and this
        // rejected every real login attempt with a 422 before the handler ran.
        let response = authenticate_by_name(
            HeaderMap::new(),
            Json(JellyfinBody::new(serde_json::json!({
                "Username": "atlas",
                "Pw": "token-abc",
                "Password": "token-abc"
            }))),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn any_username_is_accepted_because_infuse_demands_the_field() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-emby-authorization",
            HeaderValue::from_static(
                r#"MediaBrowser Client="Infuse-Direct", Device="Apple TV", DeviceId="d1", Version="8.4""#,
            ),
        );

        let response = authenticate_by_name(
            headers,
            Json(JellyfinBody::new(serde_json::json!({
                "Username": "anything at all",
                "Pw": "token-abc"
            }))),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn views_expose_one_library_per_collection_type() {
        let result = views(auth_context(), Path("ignored".to_string())).await.0;

        assert_eq!(result.total_record_count, 2);
        let types: Vec<_> = result
            .items
            .iter()
            .filter_map(|item| item.collection_type.clone())
            .collect();
        assert_eq!(types, vec!["movies".to_string(), "tvshows".to_string()]);
    }

    #[tokio::test]
    async fn view_ids_decode_back_to_their_library() {
        let view = library_view(Library::Shows, "server".to_string());
        let decoded = ItemId::parse(&view.id).expect("a view id must be an Atlas id");

        assert_eq!(decoded.as_library(), Some(Library::Shows));
        assert_eq!(view.item_type, "CollectionFolder");
        assert!(view.is_folder);
    }

    #[tokio::test]
    async fn the_path_user_id_is_never_trusted_as_identity() {
        let mine = views(auth_context(), Path("someone-elses-id".to_string()))
            .await
            .0;

        // Identity comes from the token, so a forged path changes nothing.
        assert_eq!(mine.total_record_count, 2);
    }
}
