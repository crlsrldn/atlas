use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub torbox_api_key: String,
    pub real_debrid_api_key: String,
    pub gemini_api_key: String,
    pub max_resolution: String,
    pub prefer_hdr: bool,
    pub exclude_av1: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicUserPreferences {
    pub torbox_api_key: String,
    pub real_debrid_api_key: String,
    pub gemini_api_key: String,
    pub has_torbox_api_key: bool,
    pub has_real_debrid_api_key: bool,
    pub has_gemini_api_key: bool,
    pub max_resolution: String,
    pub prefer_hdr: bool,
    pub exclude_av1: bool,
}

impl From<UserPreferences> for PublicUserPreferences {
    fn from(prefs: UserPreferences) -> Self {
        Self {
            torbox_api_key: String::new(),
            real_debrid_api_key: String::new(),
            gemini_api_key: String::new(),
            has_torbox_api_key: !prefs.torbox_api_key.is_empty(),
            has_real_debrid_api_key: !prefs.real_debrid_api_key.is_empty(),
            has_gemini_api_key: !prefs.gemini_api_key.is_empty(),
            max_resolution: prefs.max_resolution,
            prefer_hdr: prefs.prefer_hdr,
            exclude_av1: prefs.exclude_av1,
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            torbox_api_key: "".to_string(),
            real_debrid_api_key: "".to_string(),
            gemini_api_key: "".to_string(),
            max_resolution: "4K".to_string(),
            prefer_hdr: true,
            exclude_av1: false,
        }
    }
}

static PREFERENCES: Lazy<Arc<Mutex<UserPreferences>>> = Lazy::new(|| {
    Arc::new(Mutex::new(UserPreferences::default()))
});

pub async fn init_preferences() {
    if let Some(cloud_prefs) = crate::api::cloud::get_preferences_from_cloud().await {
        let mut prefs = PREFERENCES.lock().unwrap();
        *prefs = cloud_prefs;
        tracing::info!("Loaded preferences from Appwrite Cloud.");
    } else {
        // Try to load from local disk as a fallback migration
        if let Ok(data) = fs::read_to_string("preferences.json") {
            if let Ok(local_prefs) = serde_json::from_str::<UserPreferences>(&data) {
                let mut prefs = PREFERENCES.lock().unwrap();
                *prefs = local_prefs.clone();
                tracing::info!("Loaded preferences from local disk. Migrating to Cloud...");
                // Migrate to cloud
                let _ = crate::api::cloud::save_preferences_to_cloud(&local_prefs).await;
            }
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

async fn update_preferences(Json(mut payload): Json<UserPreferences>) -> Json<PublicUserPreferences> {
    {
        let mut prefs = PREFERENCES.lock().unwrap();
        if payload.torbox_api_key.is_empty() {
            payload.torbox_api_key = prefs.torbox_api_key.clone();
        }
        if payload.real_debrid_api_key.is_empty() {
            payload.real_debrid_api_key = prefs.real_debrid_api_key.clone();
        }
        if payload.gemini_api_key.is_empty() {
            payload.gemini_api_key = prefs.gemini_api_key.clone();
        }
        *prefs = payload.clone();
    }
    
    // Save to Cloud
    if let Err(e) = crate::api::cloud::save_preferences_to_cloud(&payload).await {
        tracing::error!("Failed to save preferences to Appwrite: {}", e);
    }
    
    // Save to disk as a local backup
    if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
        let _ = fs::write("preferences.json", json_str);
    }
    
    Json(payload.into())
}
