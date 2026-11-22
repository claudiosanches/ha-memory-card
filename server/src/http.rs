//! Outgoing HTTP.

use std::time::Duration;

pub fn client(timeout: Duration) -> reqwest::Client {
    builder()
        .timeout(timeout)
        .build()
        .expect("HTTP client settings are static")
}

/// One client for Home Assistant, the Supervisor and the barcode database,
/// so connections are reused. Requests set their own timeout. Redirects are
/// not followed, so a token never reaches another address.
pub fn shared() -> reqwest::Client {
    builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP client settings are static")
}

fn builder() -> reqwest::ClientBuilder {
    // reqwest is built without a crypto provider; an error means one is installed.
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder().user_agent(concat!("memory-card/", env!("CARGO_PKG_VERSION")))
}
