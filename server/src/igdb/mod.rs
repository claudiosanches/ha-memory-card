//! IGDB API client. All requests share one queue at IGDB's limit of four per
//! second; a 401 refreshes the Twitch token and a 429 waits, each retried once.

pub mod store;
pub mod sync;

use std::time::Duration;

use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tokio::time::Instant;
use utoipa::ToSchema;

use crate::config::IgdbCredentials;

const INTERVAL: Duration = Duration::from_millis(250);

/// Everything `store` needs about a game, plus `platforms` for title matching.
const GAME_FIELDS: &str = "name,summary,first_release_date,platforms,genres.name,\
    game_modes.name,game_modes.slug,involved_companies.company.name,\
    involved_companies.developer,involved_companies.publisher,cover.image_id,\
    multiplayer_modes.platform,multiplayer_modes.offlinemax,multiplayer_modes.offlinecoopmax,\
    multiplayer_modes.onlinemax,multiplayer_modes.onlinecoopmax,\
    multiplayer_modes.offlinecoop,multiplayer_modes.onlinecoop,\
    release_dates.platform,release_dates.date,release_dates.release_region,\
    game_localizations.name,game_localizations.region,game_localizations.cover.image_id";

pub struct Endpoints {
    pub token_url: String,
    pub api_url: String,
    pub image_url: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            token_url: "https://id.twitch.tv/oauth2/token".into(),
            api_url: "https://api.igdb.com/v4".into(),
            image_url: "https://images.igdb.com/igdb/image/upload".into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IgdbError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("IGDB answered {0}")]
    Status(StatusCode),
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchHit {
    pub igdb_id: i64,
    pub name: String,
    pub year: Option<i32>,
    /// Abbreviations where IGDB has them.
    pub platforms: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GameData {
    pub id: i64,
    pub name: String,
    pub summary: Option<String>,
    pub first_release_date: Option<i64>,
    #[serde(default)]
    pub platforms: Vec<i64>,
    #[serde(default)]
    pub genres: Vec<Named>,
    #[serde(default)]
    pub game_modes: Vec<Named>,
    #[serde(default)]
    pub involved_companies: Vec<InvolvedCompany>,
    pub cover: Option<Cover>,
    #[serde(default)]
    pub multiplayer_modes: Vec<MultiplayerMode>,
    #[serde(default)]
    pub release_dates: Vec<ReleaseDate>,
    #[serde(default)]
    pub game_localizations: Vec<Localization>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseDate {
    pub platform: Option<i64>,
    pub date: Option<i64>,
    /// IGDB release region id: 1 Europe, 2 North America, 8 worldwide...
    pub release_region: Option<i64>,
}

/// A regional title and cover. IGDB has them for Korea (2), Japan (3) and Europe (4).
#[derive(Debug, Deserialize)]
pub struct Localization {
    pub name: Option<String>,
    pub region: Option<i64>,
    pub cover: Option<Cover>,
}

#[derive(Debug, Deserialize)]
pub struct Named {
    pub id: i64,
    pub name: String,
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InvolvedCompany {
    pub company: Option<Company>,
    #[serde(default)]
    pub developer: bool,
    #[serde(default)]
    pub publisher: bool,
}

#[derive(Debug, Deserialize)]
pub struct Company {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Cover {
    pub image_id: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct MultiplayerMode {
    pub platform: Option<i64>,
    pub offlinemax: Option<i64>,
    pub offlinecoopmax: Option<i64>,
    pub onlinemax: Option<i64>,
    pub onlinecoopmax: Option<i64>,
    pub offlinecoop: Option<bool>,
    pub onlinecoop: Option<bool>,
}

#[derive(Deserialize)]
struct RawHit {
    id: i64,
    name: String,
    first_release_date: Option<i64>,
    #[serde(default)]
    platforms: Vec<PlatformName>,
    total_rating_count: Option<i64>,
}

#[derive(Deserialize)]
struct PlatformName {
    name: String,
    abbreviation: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

struct Token {
    value: String,
    expires_at: Instant,
}

pub struct Igdb {
    http: reqwest::Client,
    credentials: IgdbCredentials,
    endpoints: Endpoints,
    token: Mutex<Option<Token>>,
    next_request: Mutex<Instant>,
}

impl Igdb {
    pub fn new(credentials: IgdbCredentials, endpoints: Endpoints) -> Self {
        // reqwest is built without a crypto provider; an error means one is installed.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = reqwest::Client::builder()
            .user_agent(concat!("memory-card/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("HTTP client settings are static");
        Self {
            http,
            credentials,
            endpoints,
            token: Mutex::new(None),
            next_request: Mutex::new(Instant::now()),
        }
    }

    pub async fn search(&self, text: &str) -> Result<Vec<SearchHit>, IgdbError> {
        let body = format!(
            "search \"{}\"; fields name,first_release_date,platforms.name,platforms.abbreviation,total_rating_count; limit 20;",
            escape(text)
        );
        let mut hits: Vec<RawHit> = self.post("games", &body).await?;
        // IGDB ranks fan hacks next to the real game; rated games come first.
        hits.sort_by_key(|hit| std::cmp::Reverse(hit.total_rating_count.unwrap_or(0)));
        Ok(hits
            .into_iter()
            .map(|hit| SearchHit {
                igdb_id: hit.id,
                name: hit.name,
                year: hit.first_release_date.and_then(year),
                platforms: hit
                    .platforms
                    .into_iter()
                    .map(|p| p.abbreviation.unwrap_or(p.name))
                    .collect(),
            })
            .collect())
    }

    pub async fn games(&self, ids: &[i64]) -> Result<Vec<GameData>, IgdbError> {
        let ids: Vec<String> = ids.iter().map(i64::to_string).collect();
        let body = format!(
            "fields {GAME_FIELDS}; where id = ({}); limit 500;",
            ids.join(",")
        );
        self.post("games", &body).await
    }

    /// Games named exactly `name`, ignoring case.
    pub async fn named(&self, name: &str) -> Result<Vec<GameData>, IgdbError> {
        let body = format!(
            "fields {GAME_FIELDS}; where name ~ \"{}\"; limit 50;",
            escape(name)
        );
        let games: Vec<GameData> = self.post("games", &body).await?;
        let name = name.to_lowercase();
        Ok(games
            .into_iter()
            .filter(|g| g.name.to_lowercase() == name)
            .collect())
    }

    pub async fn cover(&self, image_id: &str) -> Result<Vec<u8>, IgdbError> {
        let url = format!("{}/t_cover_big_2x/{image_id}.jpg", self.endpoints.image_url);
        let response = self.http.get(url).send().await?.error_for_status()?;
        Ok(response.bytes().await?.to_vec())
    }

    async fn post<T: DeserializeOwned>(&self, endpoint: &str, body: &str) -> Result<T, IgdbError> {
        let mut retried = false;
        loop {
            let token = self.token().await?;
            self.wait_turn().await;
            let response = self
                .http
                .post(format!("{}/{endpoint}", self.endpoints.api_url))
                .header("Client-ID", &self.credentials.client_id)
                .bearer_auth(&token)
                .body(body.to_owned())
                .send()
                .await?;
            let status = response.status();
            if status.is_success() {
                return Ok(response.json().await?);
            }
            let retryable = matches!(
                status,
                StatusCode::UNAUTHORIZED | StatusCode::TOO_MANY_REQUESTS
            );
            if retried || !retryable {
                return Err(IgdbError::Status(status));
            }
            retried = true;
            if status == StatusCode::UNAUTHORIZED {
                *self.token.lock().await = None;
            } else {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn token(&self) -> Result<String, IgdbError> {
        let mut token = self.token.lock().await;
        if let Some(current) = token.as_ref().filter(|t| t.expires_at > Instant::now()) {
            return Ok(current.value.clone());
        }
        let response: TokenResponse = self
            .http
            .post(&self.endpoints.token_url)
            .form(&[
                ("client_id", self.credentials.client_id.as_str()),
                ("client_secret", self.credentials.client_secret.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let lifetime = Duration::from_secs(response.expires_in.saturating_sub(60));
        *token = Some(Token {
            value: response.access_token.clone(),
            expires_at: Instant::now() + lifetime,
        });
        Ok(response.access_token)
    }

    async fn wait_turn(&self) {
        let mut next = self.next_request.lock().await;
        tokio::time::sleep_until(*next).await;
        *next = Instant::now() + INTERVAL;
    }
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn year(timestamp: i64) -> Option<i32> {
    OffsetDateTime::from_unix_timestamp(timestamp)
        .ok()
        .map(|d| d.year())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_quotes_and_backslashes() {
        assert_eq!(escape(r#"Say "hi" \o/"#), r#"Say \"hi\" \\o/"#);
    }
}
