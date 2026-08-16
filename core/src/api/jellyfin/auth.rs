//! Request identity for the Jellyfin surface.
//!
//! Atlas has one credential: the install token. Infuse is configured with any
//! username and the token as the password, and thereafter sends the token back
//! as `X-Emby-Token` or inside `X-Emby-Authorization`.
//!
//! The token is *validated* at the gateway, which owns the Supabase lookup and
//! already resolves entitlement server-side. Core trusts the headers the gateway
//! injects and falls back to reading the client's own headers so the surface can
//! be exercised directly in development. This matches the rest of the core API,
//! which is not internet-facing.
//!
//! Authorisation is decided from the token alone. The `{userId}` that appears in
//! Jellyfin paths is derived from it and is never read back as an input — a
//! client can put anything there.

use crate::api::config::UserPreferences;
use crate::api::jellyfin::ua::ClientMode;
use crate::api::jellyfin::{server_id, stable_hex_id};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

/// Injected by the gateway once it has resolved the install token.
const HEADER_TOKEN: &str = "x-atlas-token";
const HEADER_PREFS: &str = "x-atlas-prefs";
const HEADER_PROFILE: &str = "x-atlas-profile-name";
const HEADER_MONETIZATION: &str = "x-atlas-monetization";

/// Sent by Jellyfin clients themselves.
const HEADER_EMBY_TOKEN: &str = "x-emby-token";
const HEADER_MEDIABROWSER_TOKEN: &str = "x-mediabrowser-token";
const HEADER_EMBY_AUTHORIZATION: &str = "x-emby-authorization";

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub token: String,
    pub prefs: UserPreferences,
    pub profile_name: String,
    /// Forwarded by the gateway so ranking behaves identically on both
    /// surfaces; it is an input to `rank_sources`.
    pub monetization_enabled: bool,
    pub client: Option<String>,
    pub device: Option<String>,
    pub device_id: Option<String>,
    pub version: Option<String>,
    pub user_agent: Option<String>,
}

impl AuthContext {
    /// Stable, non-reversible, and derived rather than stored — so it survives
    /// restarts without a table, and never exposes the token it came from.
    pub fn user_id(&self) -> String {
        stable_hex_id("atlas-user", &self.token)
    }

    pub fn server_id(&self) -> String {
        server_id()
    }

    /// A session id that is stable for one client on one device.
    pub fn session_id(&self) -> String {
        let device = self.device_id.as_deref().unwrap_or("unknown-device");
        stable_hex_id("atlas-session", &format!("{}:{}", self.token, device))
    }

    /// Prefer the client name Infuse declares in `X-Emby-Authorization` over the
    /// User-Agent: it is the field Infuse actually varies by mode.
    pub fn mode(&self) -> ClientMode {
        let declared = self
            .client
            .as_deref()
            .map(|client| ClientMode::from_user_agent(Some(client)));

        match declared {
            Some(ClientMode::Other) | None => {
                ClientMode::from_user_agent(self.user_agent.as_deref())
            }
            Some(mode) => mode,
        }
    }

    /// The device name Infuse reports ("Apple TV"), which is a better capability
    /// signal than the User-Agent — `ai_decision::infer_capabilities` matches
    /// none of its rules against `Infuse-Direct/7.7`.
    pub fn device_hint(&self) -> String {
        self.device
            .clone()
            .unwrap_or_else(|| self.user_agent.clone().unwrap_or_default())
    }

    /// The device name rewritten into the shape `ai_decision` looks for.
    ///
    /// Those rules match User-Agent fragments like `appletv`, while Infuse
    /// reports a human name with a space in it, so "Apple TV" would slip past
    /// the very rule written for it.
    pub fn capability_hint(&self) -> String {
        let device = self
            .device
            .clone()
            .unwrap_or_default()
            .to_lowercase()
            .replace([' ', '-', '_'], "");
        let agent = self.user_agent.clone().unwrap_or_default().to_lowercase();

        format!("{device} {agent}")
    }
}

pub struct AuthRejection;

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "Error": "Unauthorized",
                "Message": "Provide the Atlas install token as the password."
            })),
        )
            .into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = |name: &str| {
            parts
                .headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
        };

        let authorization = header(HEADER_EMBY_AUTHORIZATION)
            .or_else(|| header("authorization"))
            .unwrap_or_default();
        let declared = parse_authorization(&authorization);

        let token = header(HEADER_TOKEN)
            .or_else(|| header(HEADER_EMBY_TOKEN))
            .or_else(|| header(HEADER_MEDIABROWSER_TOKEN))
            .or_else(|| declared.token.clone())
            .filter(|token| !token.is_empty())
            .ok_or(AuthRejection)?;

        // The gateway resolves preferences and sends them with the token.
        //
        // Defaulting when the header is absent is deliberate and fails closed:
        // default preferences carry no provider credential, so a request that
        // somehow bypassed the gateway resolves nothing rather than borrowing
        // whatever key the server itself holds.
        //
        // Permissive mode is the exception, and only that. It is the
        // development flag — off in production and in CI — and there it falls
        // back to the server's own preferences so the surface can be exercised
        // without Supabase in the path.
        let prefs = header(HEADER_PREFS)
            .and_then(|raw| serde_json::from_str::<UserPreferences>(&raw).ok())
            .unwrap_or_else(|| {
                if super::permissive() {
                    crate::api::config::current_preferences()
                } else {
                    UserPreferences::default()
                }
            });

        Ok(AuthContext {
            token,
            prefs,
            profile_name: header(HEADER_PROFILE).unwrap_or_else(|| "Atlas".to_string()),
            monetization_enabled: matches!(header(HEADER_MONETIZATION).as_deref(), Some("true")),
            client: declared.client,
            device: declared.device,
            device_id: declared.device_id,
            version: declared.version,
            user_agent: header("user-agent"),
        })
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DeclaredClient {
    pub client: Option<String>,
    pub device: Option<String>,
    pub device_id: Option<String>,
    pub version: Option<String>,
    pub token: Option<String>,
}

/// Parses the Emby authorization header, which looks like:
///
/// ```text
/// MediaBrowser Client="Infuse-Direct", Device="Apple TV", DeviceId="…", Version="8.4", Token="…"
/// ```
pub fn parse_authorization(value: &str) -> DeclaredClient {
    let mut declared = DeclaredClient::default();

    let body = value
        .split_once(char::is_whitespace)
        .map(|(_scheme, rest)| rest)
        .unwrap_or(value);

    for pair in body.split(',') {
        let Some((key, raw)) = pair.split_once('=') else {
            continue;
        };
        let field = raw.trim().trim_matches('"').trim();
        if field.is_empty() {
            continue;
        }

        match key.trim().to_ascii_lowercase().as_str() {
            "client" => declared.client = Some(field.to_string()),
            "device" => declared.device = Some(field.to_string()),
            "deviceid" => declared.device_id = Some(field.to_string()),
            "version" => declared.version = Some(field.to_string()),
            "token" => declared.token = Some(field.to_string()),
            _ => {}
        }
    }

    declared
}

#[cfg(test)]
mod tests {
    use super::{parse_authorization, AuthContext};
    use crate::api::config::UserPreferences;
    use crate::api::jellyfin::ua::ClientMode;

    fn context(client: Option<&str>, user_agent: Option<&str>) -> AuthContext {
        AuthContext {
            token: "token-abc".to_string(),
            prefs: UserPreferences::default(),
            profile_name: "Atlas".to_string(),
            monetization_enabled: false,
            client: client.map(str::to_string),
            device: Some("Apple TV".to_string()),
            device_id: Some("device-1".to_string()),
            version: Some("8.4".to_string()),
            user_agent: user_agent.map(str::to_string),
        }
    }

    #[test]
    fn parses_the_emby_authorization_header() {
        let declared = parse_authorization(
            r#"MediaBrowser Client="Infuse-Direct", Device="Apple TV", DeviceId="abc123", Version="8.4", Token="secret""#,
        );

        assert_eq!(declared.client.as_deref(), Some("Infuse-Direct"));
        assert_eq!(declared.device.as_deref(), Some("Apple TV"));
        assert_eq!(declared.device_id.as_deref(), Some("abc123"));
        assert_eq!(declared.version.as_deref(), Some("8.4"));
        assert_eq!(declared.token.as_deref(), Some("secret"));
    }

    #[test]
    fn authorization_parsing_tolerates_missing_and_odd_fields() {
        let declared =
            parse_authorization(r#"MediaBrowser Client="Swiftfin", Unknown="x", Token="""#);

        assert_eq!(declared.client.as_deref(), Some("Swiftfin"));
        assert_eq!(declared.device, None);
        // An empty Token= must not be mistaken for a credential.
        assert_eq!(declared.token, None);
    }

    #[test]
    fn the_declared_client_decides_the_mode_over_the_user_agent() {
        // Infuse varies Client by connection mode, so it is the better signal.
        let context = context(Some("Infuse-Library"), Some("Infuse-Direct/7.7"));

        assert_eq!(context.mode(), ClientMode::InfuseLibrary);
    }

    #[test]
    fn falls_back_to_the_user_agent_when_no_client_is_declared() {
        assert_eq!(
            context(None, Some("Infuse-Direct/7.7")).mode(),
            ClientMode::InfuseDirect
        );
        assert_eq!(
            context(Some("Swiftfin"), Some("Infuse-Library/7.7")).mode(),
            ClientMode::InfuseLibrary
        );
    }

    #[test]
    fn identities_are_derived_stably_and_hide_the_token() {
        let context = context(Some("Infuse-Direct"), None);

        assert_eq!(context.user_id(), context.user_id());
        assert_eq!(context.user_id().len(), 32);
        assert!(!context.user_id().contains("token-abc"));
        assert!(!context.session_id().contains("token-abc"));
    }

    #[test]
    fn sessions_separate_devices_sharing_one_token() {
        let apple_tv = context(None, None);
        let mut ipad = apple_tv.clone();
        ipad.device_id = Some("device-2".to_string());

        assert_eq!(apple_tv.user_id(), ipad.user_id());
        assert_ne!(apple_tv.session_id(), ipad.session_id());
    }

    #[test]
    fn prefers_the_declared_device_as_the_capability_hint() {
        assert_eq!(
            context(None, Some("Infuse-Direct/7.7")).device_hint(),
            "Apple TV"
        );
    }
}
