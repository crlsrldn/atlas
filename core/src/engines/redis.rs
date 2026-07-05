use redis::aio::ConnectionManager;
use redis::Client;
use std::env;
use std::sync::OnceLock;

static REDIS_CLIENT: OnceLock<ConnectionManager> = OnceLock::new();

pub async fn init_redis() {
    let redis_url =
        env::var("UPSTASH_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // We only initialize if it's set properly, or we fallback gracefully
    if let Ok(client) = Client::open(redis_url) {
        if let Ok(manager) = client.get_connection_manager().await {
            let _ = REDIS_CLIENT.set(manager);
            println!("✅ Redis caching initialized");
        } else {
            println!("⚠️ Failed to create Redis connection manager");
        }
    } else {
        println!("⚠️ Failed to parse Redis URL");
    }
}

pub fn get_redis() -> Option<ConnectionManager> {
    REDIS_CLIENT.get().cloned()
}
