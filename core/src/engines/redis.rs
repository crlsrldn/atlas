use redis::aio::ConnectionManager;
use redis::Client;
use std::env;
use std::sync::OnceLock;

static REDIS_CLIENT: OnceLock<ConnectionManager> = OnceLock::new();

pub async fn init_redis() {
    let configured = env::var("UPSTASH_REDIS_URL").ok();
    if configured.is_none() {
        // Worth saying out loud: without Redis, source results are cached only
        // in-process, so each machine keeps its own copy.
        tracing::warn!(
            "UPSTASH_REDIS_URL is not set — falling back to the in-process source cache"
        );
    }

    let redis_url = configured.unwrap_or_else(|| "redis://127.0.0.1:6379".to_string());

    let Ok(client) = Client::open(redis_url) else {
        tracing::warn!("Failed to parse Redis URL — using the in-process source cache");
        return;
    };

    match client.get_connection_manager().await {
        Ok(manager) => {
            let _ = REDIS_CLIENT.set(manager);
            tracing::info!("Redis caching initialized");
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Could not connect to Redis — using the in-process source cache"
            );
        }
    }
}

pub fn get_redis() -> Option<ConnectionManager> {
    REDIS_CLIENT.get().cloned()
}
