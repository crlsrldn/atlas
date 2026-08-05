use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .expect("failed to build shared HTTP client")
});

pub fn client() -> &'static Client {
    &HTTP_CLIENT
}
