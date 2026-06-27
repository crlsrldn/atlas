use crate::api::config::{PublicUserPreferences, UserPreferences};
use crate::api::vault::{open_secret, seal_secret, SecretHandle};
use axum::{
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const DEFAULT_USER_ID: &str = "demo-user";
const DEFAULT_PROFILE_ID: &str = "default";
const DEFAULT_INSTALL_TOKEN: &str = "demo-install-token";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Free,
    Trialing,
    Active,
    PastDue,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    Free,
    Pro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantRecord {
    pub user_id: String,
    pub profile_id: String,
    pub install_token: String,
    pub plan: Plan,
    pub subscription_status: SubscriptionStatus,
    pub monthly_resolve_quota: u32,
    pub monthly_resolve_count: u32,
    pub preferences: UserPreferences,
    pub torbox_secret: SecretHandle,
    pub real_debrid_secret: SecretHandle,
    pub gemini_secret: SecretHandle,
}

#[derive(Debug, Serialize)]
pub struct PublicTenant {
    pub user_id: String,
    pub profile_id: String,
    pub install_token: String,
    pub plan: Plan,
    pub subscription_status: SubscriptionStatus,
    pub monthly_resolve_quota: u32,
    pub monthly_resolve_count: u32,
    pub stremio_manifest_path: String,
    pub preferences: PublicUserPreferences,
}

#[derive(Debug, Deserialize)]
struct SessionRequest {
    user_id: Option<String>,
    profile_id: Option<String>,
}

static TENANTS: Lazy<Arc<Mutex<HashMap<String, TenantRecord>>>> = Lazy::new(|| {
    let mut tenants = HashMap::new();
    let tenant = TenantRecord::default_demo();
    tenants.insert(tenant.user_id.clone(), tenant);
    Arc::new(Mutex::new(tenants))
});

impl TenantRecord {
    pub fn default_demo() -> Self {
        let preferences = UserPreferences::default();
        Self {
            user_id: DEFAULT_USER_ID.to_string(),
            profile_id: DEFAULT_PROFILE_ID.to_string(),
            install_token: DEFAULT_INSTALL_TOKEN.to_string(),
            plan: Plan::Free,
            subscription_status: SubscriptionStatus::Free,
            monthly_resolve_quota: 50,
            monthly_resolve_count: 0,
            preferences,
            torbox_secret: SecretHandle::empty("torbox"),
            real_debrid_secret: SecretHandle::empty("real_debrid"),
            gemini_secret: SecretHandle::empty("gemini"),
        }
    }

    pub fn hydrated_preferences(&self) -> UserPreferences {
        let mut preferences = self.preferences.clone();
        preferences.torbox_api_key = open_secret(&self.torbox_secret);
        preferences.real_debrid_api_key = open_secret(&self.real_debrid_secret);
        preferences.gemini_api_key = open_secret(&self.gemini_secret);
        preferences
    }

    pub fn public(&self) -> PublicTenant {
        let mut public_prefs: PublicUserPreferences = self.preferences.clone().into();
        public_prefs.has_torbox_api_key = self.torbox_secret.is_configured();
        public_prefs.has_real_debrid_api_key = self.real_debrid_secret.is_configured();
        public_prefs.has_gemini_api_key = self.gemini_secret.is_configured();

        PublicTenant {
            user_id: self.user_id.clone(),
            profile_id: self.profile_id.clone(),
            install_token: self.install_token.clone(),
            plan: self.plan.clone(),
            subscription_status: self.subscription_status.clone(),
            monthly_resolve_quota: self.monthly_resolve_quota,
            monthly_resolve_count: self.monthly_resolve_count,
            stremio_manifest_path: format!("/stremio/{}/manifest.json", self.install_token),
            preferences: public_prefs,
        }
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/auth/session", post(create_session))
        .route("/v1/account", get(account))
        .route("/v1/preferences", get(preferences))
        .route("/v1/preferences", post(update_preferences))
}

async fn create_session(Json(payload): Json<SessionRequest>) -> Json<PublicTenant> {
    let user_id = payload
        .user_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_USER_ID.to_string());
    let profile_id = payload
        .profile_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_PROFILE_ID.to_string());

    let mut tenants = TENANTS.lock().unwrap();
    let tenant = tenants
        .entry(user_id.clone())
        .or_insert_with(|| TenantRecord {
            user_id: user_id.clone(),
            profile_id: profile_id.clone(),
            install_token: generate_install_token(&user_id, &profile_id),
            ..TenantRecord::default_demo()
        });
    tenant.profile_id = profile_id;

    Json(tenant.public())
}

async fn account(headers: HeaderMap) -> Json<PublicTenant> {
    Json(current_tenant(&headers).public())
}

async fn preferences(headers: HeaderMap) -> Json<PublicUserPreferences> {
    Json(current_tenant(&headers).public().preferences)
}

async fn update_preferences(
    headers: HeaderMap,
    Json(mut payload): Json<UserPreferences>,
) -> Json<PublicUserPreferences> {
    let user_id = user_id_from_headers(&headers);
    let mut tenants = TENANTS.lock().unwrap();
    let tenant = tenants
        .entry(user_id.clone())
        .or_insert_with(|| TenantRecord {
            user_id: user_id.clone(),
            install_token: generate_install_token(&user_id, DEFAULT_PROFILE_ID),
            ..TenantRecord::default_demo()
        });

    if !payload.torbox_api_key.is_empty() {
        tenant.torbox_secret = seal_secret("torbox", &payload.torbox_api_key);
    }
    if !payload.real_debrid_api_key.is_empty() {
        tenant.real_debrid_secret = seal_secret("real_debrid", &payload.real_debrid_api_key);
    }
    if !payload.gemini_api_key.is_empty() {
        tenant.gemini_secret = seal_secret("gemini", &payload.gemini_api_key);
    }

    payload.torbox_api_key.clear();
    payload.real_debrid_api_key.clear();
    payload.gemini_api_key.clear();
    tenant.preferences = payload;

    Json(tenant.public().preferences)
}

pub fn current_tenant(headers: &HeaderMap) -> TenantRecord {
    let user_id = user_id_from_headers(headers);
    tenant_by_user_id(&user_id).unwrap_or_else(TenantRecord::default_demo)
}

pub fn tenant_by_user_id(user_id: &str) -> Option<TenantRecord> {
    TENANTS.lock().unwrap().get(user_id).cloned()
}

pub fn tenant_by_install_token(token: &str) -> Option<TenantRecord> {
    TENANTS
        .lock()
        .unwrap()
        .values()
        .find(|tenant| tenant.install_token == token)
        .cloned()
}

pub fn charge_resolve(token: &str) -> Result<TenantRecord, &'static str> {
    let mut tenants = TENANTS.lock().unwrap();
    let Some((_, tenant)) = tenants
        .iter_mut()
        .find(|(_, tenant)| tenant.install_token == token)
    else {
        return Err("invalid_install_token");
    };

    if tenant.plan == Plan::Free && tenant.monthly_resolve_count >= tenant.monthly_resolve_quota {
        return Err("quota_exceeded");
    }

    tenant.monthly_resolve_count += 1;
    Ok(tenant.clone())
}

pub fn mark_subscription(user_id: &str, plan: Plan, status: SubscriptionStatus) -> PublicTenant {
    let mut tenants = TENANTS.lock().unwrap();
    let tenant = tenants
        .entry(user_id.to_string())
        .or_insert_with(|| TenantRecord {
            user_id: user_id.to_string(),
            install_token: generate_install_token(user_id, DEFAULT_PROFILE_ID),
            ..TenantRecord::default_demo()
        });
    tenant.plan = plan;
    tenant.subscription_status = status;
    tenant.monthly_resolve_quota = if tenant.plan == Plan::Pro { 5_000 } else { 50 };
    tenant.public()
}

fn user_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-atlas-user-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(DEFAULT_USER_ID)
        .to_string()
}

fn generate_install_token(user_id: &str, profile_id: &str) -> String {
    let mut hash: u64 = 14_695_981_039_346_656_037;
    for byte in format!("{}:{}", user_id, profile_id).as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    format!("atl_{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::{charge_resolve, tenant_by_install_token, TenantRecord};

    #[test]
    fn demo_install_token_maps_to_one_tenant() {
        let tenant = TenantRecord::default_demo();
        let loaded = tenant_by_install_token(&tenant.install_token).unwrap();

        assert_eq!(loaded.user_id, tenant.user_id);
    }

    #[test]
    fn free_tenant_resolve_counter_is_scoped_to_install_token() {
        let tenant = charge_resolve("demo-install-token").unwrap();

        assert_eq!(tenant.user_id, "demo-user");
        assert!(tenant.monthly_resolve_count >= 1);
    }
}
