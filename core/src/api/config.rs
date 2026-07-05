use axum::{
    routing::{get, post},
    Json, Router,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    #[serde(default)]
    pub torbox_api_key: String,
    #[serde(default)]
    pub gemini_api_key: String,
    #[serde(default)]
    pub trakt_client_id: String,
    #[serde(default)]
    pub trakt_username: String,
    #[serde(default = "default_max_resolution")]
    pub max_resolution: String,
    #[serde(default = "default_prefer_hdr")]
    pub prefer_hdr: bool,
    #[serde(default)]
    pub exclude_av1: bool,
    #[serde(default)]
    pub exclude_hevc: bool,
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default)]
    pub mobile_data_saver: bool,
    #[serde(default)]
    pub home_theater_mode: bool,
    #[serde(default)]
    pub family_mode: bool,
    #[serde(default = "default_language")]
    pub preferred_language: String,
    #[serde(default = "default_subtitle_mode")]
    pub subtitle_mode: String,
    #[serde(default = "default_sort_preference")]
    pub sort_preference: String,
    #[serde(default = "default_stream_limit")]
    pub stream_limit: u32,
    #[serde(default)]
    pub is_premium: bool,
    #[serde(default)]
    pub max_size_gb: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicUserPreferences {
    pub torbox_api_key: String,
    pub gemini_api_key: String,
    pub trakt_client_id: String,
    pub trakt_username: String,
    pub has_torbox_api_key: bool,
    pub has_gemini_api_key: bool,
    pub has_trakt_client_id: bool,
    pub has_trakt_username: bool,
    pub max_resolution: String,
    pub prefer_hdr: bool,
    pub exclude_av1: bool,
    pub exclude_hevc: bool,
    pub profile: String,
    pub mobile_data_saver: bool,
    pub home_theater_mode: bool,
    pub family_mode: bool,
    pub preferred_language: String,
    pub subtitle_mode: String,
    pub sort_preference: String,
    pub stream_limit: u32,
    pub is_premium: bool,
    pub max_size_gb: Option<u32>,
}

impl From<UserPreferences> for PublicUserPreferences {
    fn from(prefs: UserPreferences) -> Self {
        Self {
            torbox_api_key: String::new(),
            gemini_api_key: String::new(),
            trakt_client_id: String::new(),
            trakt_username: prefs.trakt_username.clone(), // public is fine
            has_torbox_api_key: !prefs.torbox_api_key.is_empty(),
            has_gemini_api_key: !prefs.gemini_api_key.is_empty(),
            has_trakt_client_id: !prefs.trakt_client_id.is_empty(),
            has_trakt_username: !prefs.trakt_username.is_empty(),
            max_resolution: prefs.max_resolution,
            prefer_hdr: prefs.prefer_hdr,
            exclude_av1: prefs.exclude_av1,
            exclude_hevc: prefs.exclude_hevc,
            profile: prefs.profile,
            mobile_data_saver: prefs.mobile_data_saver,
            home_theater_mode: prefs.home_theater_mode,
            family_mode: prefs.family_mode,
            preferred_language: prefs.preferred_language,
            subtitle_mode: prefs.subtitle_mode,
            sort_preference: prefs.sort_preference,
            stream_limit: prefs.stream_limit,
            is_premium: prefs.is_premium,
            max_size_gb: prefs.max_size_gb,
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            torbox_api_key: String::new(),
            gemini_api_key: String::new(),
            trakt_client_id: String::new(),
            trakt_username: String::new(),
            max_resolution: default_max_resolution(),
            prefer_hdr: default_prefer_hdr(),
            exclude_av1: false,
            exclude_hevc: false,
            profile: default_profile(),
            mobile_data_saver: false,
            home_theater_mode: false,
            family_mode: false,
            preferred_language: default_language(),
            subtitle_mode: default_subtitle_mode(),
            sort_preference: default_sort_preference(),
            stream_limit: default_stream_limit(),
            is_premium: false,
            max_size_gb: None,
        }
    }
}

fn default_max_resolution() -> String {
    "4K".to_string()
}

fn default_prefer_hdr() -> bool {
    true
}

fn default_profile() -> String {
    "home_theater".to_string()
}

fn default_language() -> String {
    "English".to_string()
}

fn default_subtitle_mode() -> String {
    "auto".to_string()
}

fn default_sort_preference() -> String {
    "balanced".to_string()
}

fn default_stream_limit() -> u32 {
    5
}

impl UserPreferences {
    pub fn without_secrets(&self) -> Self {
        let mut prefs = self.clone();
        prefs.torbox_api_key.clear();
        prefs.gemini_api_key.clear();
        prefs
    }
}

static PREFERENCES: Lazy<Arc<Mutex<UserPreferences>>> =
    Lazy::new(|| Arc::new(Mutex::new(UserPreferences::default())));

pub async fn init_preferences() {
    if let Some(mut cloud_prefs) = crate::api::cloud::get_preferences_from_cloud().await {
        migrate_secrets_to_keychain(&cloud_prefs);
        hydrate_secrets(&mut cloud_prefs);
        {
            let mut prefs = PREFERENCES.lock().unwrap();
            *prefs = cloud_prefs.clone();
        }
        tracing::info!("Loaded preferences from Supabase.");
        let redacted_prefs = cloud_prefs.without_secrets();
        let _ = crate::api::cloud::save_preferences_to_cloud(&redacted_prefs).await;
        save_preferences_to_disk(&redacted_prefs);
    } else {
        // Try to load from local disk as a fallback migration
        if let Ok(data) = fs::read_to_string("preferences.json") {
            if let Ok(mut local_prefs) = serde_json::from_str::<UserPreferences>(&data) {
                migrate_secrets_to_keychain(&local_prefs);
                hydrate_secrets(&mut local_prefs);
                {
                    let mut prefs = PREFERENCES.lock().unwrap();
                    *prefs = local_prefs.clone();
                }
                tracing::info!(
                    "Loaded preferences from local disk. Migrating non-secret settings to Cloud..."
                );
                let redacted_prefs = local_prefs.without_secrets();
                let _ = crate::api::cloud::save_preferences_to_cloud(&redacted_prefs).await;
                save_preferences_to_disk(&redacted_prefs);
            }
        } else {
            let mut prefs = UserPreferences::default();
            hydrate_secrets(&mut prefs);
            let mut current = PREFERENCES.lock().unwrap();
            *current = prefs;
        }
    }
}

pub fn current_preferences() -> UserPreferences {
    PREFERENCES.lock().unwrap().clone()
}

pub fn router() -> Router {
    Router::new()
        .route("/user/preferences", get(get_preferences))
        .route("/user/preferences", post(update_preferences))
}

async fn get_preferences() -> Json<PublicUserPreferences> {
    let prefs = PREFERENCES.lock().unwrap().clone();
    Json(prefs.into())
}

async fn update_preferences(
    Json(mut payload): Json<UserPreferences>,
) -> Json<PublicUserPreferences> {
    {
        let mut prefs = PREFERENCES.lock().unwrap();
        if payload.torbox_api_key.is_empty() {
            payload.torbox_api_key = prefs.torbox_api_key.clone();
        }
        if payload.gemini_api_key.is_empty() {
            payload.gemini_api_key = prefs.gemini_api_key.clone();
        }
        persist_secrets(&payload);
        *prefs = payload.clone();
    }

    let redacted_payload = payload.without_secrets();

    // Save to Cloud
    if let Err(e) = crate::api::cloud::save_preferences_to_cloud(&redacted_payload).await {
        tracing::error!("Failed to save preferences to Supabase: {}", e);
    }

    save_preferences_to_disk(&redacted_payload);

    Json(payload.into())
}

fn hydrate_secrets(prefs: &mut UserPreferences) {
    if prefs.torbox_api_key.is_empty() {
        prefs.torbox_api_key =
            crate::api::secret_store::read_secret("torbox_api_key").unwrap_or_default();
    }
    if prefs.gemini_api_key.is_empty() {
        prefs.gemini_api_key =
            crate::api::secret_store::read_secret("gemini_api_key").unwrap_or_default();
    }
}

fn migrate_secrets_to_keychain(prefs: &UserPreferences) {
    persist_secret("torbox_api_key", &prefs.torbox_api_key);
    persist_secret("torbox_api_key", &prefs.torbox_api_key);
    persist_secret("gemini_api_key", &prefs.gemini_api_key);
}

fn persist_secrets(prefs: &UserPreferences) {
    persist_secret("torbox_api_key", &prefs.torbox_api_key);
    persist_secret("torbox_api_key", &prefs.torbox_api_key);
    persist_secret("gemini_api_key", &prefs.gemini_api_key);
}

fn persist_secret(account: &str, secret: &str) {
    if secret.is_empty() {
        return;
    }

    if let Err(e) = crate::api::secret_store::write_secret(account, secret) {
        tracing::error!("Failed to save {} to keychain: {}", account, e);
    }
}

fn save_preferences_to_disk(prefs: &UserPreferences) {
    if let Ok(json_str) = serde_json::to_string_pretty(prefs) {
        let _ = fs::write("preferences.json", json_str);
    }
}
