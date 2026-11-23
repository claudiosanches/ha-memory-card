//! Outgoing HTTP.

use std::time::Duration;

pub fn client(timeout: Duration) -> reqwest::Client {
    builder()
        .timeout(timeout)
        .build()
        .expect("HTTP client settings are static")
}

/// One client for Home Assistant, the Supervisor and the barcode database,
/// so connections are reused. Requests set their own timeout.
pub fn shared() -> reqwest::Client {
    builder().build().expect("HTTP client settings are static")
}

/// Redirects are not followed, so a token or secret never reaches another
/// address.
fn builder() -> reqwest::ClientBuilder {
    // reqwest is built without a crypto provider; an error means one is installed.
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder()
        .user_agent(concat!("memory-card/", env!("CARGO_PKG_VERSION")))
        .redirect(reqwest::redirect::Policy::none())
}

/// The body of `response`, or `None` once it passes `max` bytes.
pub async fn body(mut response: reqwest::Response, max: usize) -> reqwest::Result<Option<Vec<u8>>> {
    if response
        .content_length()
        .is_some_and(|length| length > max as u64)
    {
        return Ok(None);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > max {
            return Ok(None);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(Some(bytes))
}
