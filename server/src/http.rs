//! Outgoing HTTP.

use std::time::Duration;

pub fn client(timeout: Duration) -> reqwest::Client {
    // reqwest is built without a crypto provider; an error means one is installed.
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder()
        .user_agent(concat!("memory-card/", env!("CARGO_PKG_VERSION")))
        .timeout(timeout)
        .build()
        .expect("HTTP client settings are static")
}
