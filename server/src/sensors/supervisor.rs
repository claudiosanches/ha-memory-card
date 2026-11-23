//! What the sensors ask the Supervisor: this app's own details, and the
//! discovery message that points Home Assistant at the integration feed.

use std::time::Duration;

use reqwest::{Method, RequestBuilder, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::config::{Config, Secret};

const TIMEOUT: Duration = Duration::from_secs(10);
/// The integration's domain, listed under `discovery` in config.yaml.
pub const SERVICE: &str = "memory_card";

#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("cannot reach the Supervisor: {0}")]
    Http(#[from] reqwest::Error),
    #[error("the Supervisor answered {0}")]
    Status(StatusCode),
    #[error("the Supervisor sent an unusable discovery id")]
    BadUuid,
}

#[derive(Deserialize)]
pub(crate) struct Reply<T> {
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelfInfo {
    pub slug: String,
    /// Other containers on the Supervisor's network reach the app by this name.
    pub hostname: String,
}

#[derive(Deserialize)]
struct Announced {
    uuid: String,
}

pub(crate) fn request(
    config: &Config,
    http: &reqwest::Client,
    token: &Secret,
    method: Method,
    path: &str,
) -> RequestBuilder {
    http.request(method, format!("{}{path}", config.supervisor_url))
        .timeout(TIMEOUT)
        .bearer_auth(token.expose())
}

/// `Ok(None)` outside Home Assistant.
pub async fn self_info(
    config: &Config,
    http: &reqwest::Client,
) -> Result<Option<SelfInfo>, SupervisorError> {
    let Some(token) = &config.supervisor_token else {
        return Ok(None);
    };
    let response = request(config, http, token, Method::GET, "/addons/self/info")
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(SupervisorError::Status(response.status()));
    }
    Ok(Some(response.json::<Reply<SelfInfo>>().await?.data))
}

/// Tells Home Assistant where the feed is. Returns the message's id, which
/// stays the same for this app while it keeps sending.
pub async fn announce(
    config: &Config,
    http: &reqwest::Client,
    token: &Secret,
    service_config: Value,
) -> Result<String, SupervisorError> {
    let body = json!({ "service": SERVICE, "config": service_config });
    let response = request(config, http, token, Method::POST, "/discovery")
        .json(&body)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(SupervisorError::Status(response.status()));
    }
    let uuid = response.json::<Reply<Announced>>().await?.data.uuid;
    if valid_uuid(&uuid) {
        Ok(uuid)
    } else {
        Err(SupervisorError::BadUuid)
    }
}

/// Removes the message, so Home Assistant removes the integration's entry.
/// A message already gone counts as removed.
pub async fn withdraw(
    config: &Config,
    http: &reqwest::Client,
    token: &Secret,
    uuid: &str,
) -> Result<(), SupervisorError> {
    if !valid_uuid(uuid) {
        return Err(SupervisorError::BadUuid);
    }
    let response = request(
        config,
        http,
        token,
        Method::DELETE,
        &format!("/discovery/{uuid}"),
    )
    .send()
    .await?;
    match response.status() {
        status if status.is_success() || status == StatusCode::NOT_FOUND => Ok(()),
        status => Err(SupervisorError::Status(status)),
    }
}

/// The Supervisor's ids are 32 hex digits; anything else never reaches a URL.
fn valid_uuid(uuid: &str) -> bool {
    uuid.len() == 32 && uuid.bytes().all(|b| b.is_ascii_hexdigit())
}
