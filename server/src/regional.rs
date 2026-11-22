//! Home Assistant's language and currency, the defaults until someone picks
//! their own.

use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::AppState;

/// Wait before asking again when Home Assistant did not answer, such as
/// while it starts.
const RETRY: Duration = Duration::from_secs(60);
const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Regional {
    /// A language tag such as `pt-BR`.
    pub language: Option<String>,
    /// An ISO 4217 code.
    pub currency: Option<String>,
}

#[derive(Debug, Default)]
pub struct Cache {
    found: Option<Regional>,
    tried: Option<Instant>,
}

#[derive(Deserialize)]
struct CoreConfig {
    language: Option<String>,
    currency: Option<String>,
}

/// Home Assistant's settings, asked for once and kept. `None` outside Home
/// Assistant or while it does not answer.
pub async fn get(state: &AppState) -> Option<Regional> {
    let api = state.config.home_assistant.as_ref()?;
    let mut cache = state.regional.lock().await;
    if let Some(found) = &cache.found {
        return Some(found.clone());
    }
    if cache.tried.is_some_and(|at| at.elapsed() < RETRY) {
        return None;
    }
    cache.tried = Some(Instant::now());
    let response = state
        .http
        .get(format!("{}/config", api.rest))
        .timeout(TIMEOUT)
        .bearer_auth(api.token.expose())
        .send()
        .await;
    let config = match response {
        Ok(response) if response.status().is_success() => response.json::<CoreConfig>().await,
        Ok(response) => {
            tracing::warn!(status = %response.status(), "Home Assistant did not share its settings");
            return None;
        }
        Err(err) => Err(err),
    };
    let config = match config {
        Ok(config) => config,
        Err(err) => {
            tracing::warn!(error = %err, "cannot read Home Assistant's settings");
            return None;
        }
    };
    let found = Regional {
        language: config.language.filter(|tag| language_tag(tag)),
        currency: config.currency.as_deref().and_then(crate::money::currency),
    };
    cache.found = Some(found.clone());
    Some(found)
}

fn language_tag(tag: &str) -> bool {
    (2..=20).contains(&tag.len())
        && tag
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
