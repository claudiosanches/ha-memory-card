//! IGDB API client. All requests share one queue at IGDB's limit of four per
//! second; a 401 refreshes the Twitch token and a 429 waits, each retried once.

pub mod matches;
pub mod store;
pub mod sync;

use std::fmt;
use std::marker::PhantomData;
use std::time::Duration;

use reqwest::StatusCode;
use serde::de::{DeserializeOwned, IgnoredAny, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
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
    game_localizations.name,game_localizations.region,game_localizations.cover.image_id,\
    version_parent,remasters,remakes,ports,alternative_names.name,alternative_names.comment";

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
    #[error("IGDB's answer is larger than expected")]
    TooLarge,
    #[error("IGDB's answer is not what was asked for: {0}")]
    Json(#[from] serde_json::Error),
}

/// Covers are a few hundred KB; anything near this is not one.
const COVER_MAX: usize = 10 * 1024 * 1024;
/// 500 games with every field asked for come to a few MB.
const ANSWER_MAX: usize = 16 * 1024 * 1024;
/// Twitch's tokens last about 60 days; a longer lifetime is not believed.
const TOKEN_LIFETIME_MAX: u64 = 90 * 24 * 60 * 60;
/// Items kept from each list about a game; real games stay far below it,
/// and each item costs a write.
const LIST_MAX: usize = 500;

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchHit {
    pub igdb_id: i64,
    pub name: String,
    pub year: Option<i32>,
    /// Abbreviations where IGDB has them.
    pub platforms: Vec<String>,
    /// IGDB's ids for the same platforms.
    pub platform_ids: Vec<i64>,
    /// The game in the collection already linked to this IGDB game.
    pub game_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct GameData {
    pub id: i64,
    pub name: String,
    pub summary: Option<String>,
    pub first_release_date: Option<i64>,
    #[serde(default, deserialize_with = "capped")]
    pub platforms: Vec<i64>,
    #[serde(default, deserialize_with = "capped")]
    pub genres: Vec<Named>,
    #[serde(default, deserialize_with = "capped")]
    pub game_modes: Vec<Named>,
    #[serde(default, deserialize_with = "capped")]
    pub involved_companies: Vec<InvolvedCompany>,
    pub cover: Option<Cover>,
    #[serde(default, deserialize_with = "capped")]
    pub multiplayer_modes: Vec<MultiplayerMode>,
    #[serde(default, deserialize_with = "capped")]
    pub release_dates: Vec<ReleaseDate>,
    #[serde(default, deserialize_with = "capped")]
    pub game_localizations: Vec<Localization>,
    /// The main game when this one is an edition or other version of it.
    pub version_parent: Option<i64>,
    #[serde(default, deserialize_with = "capped")]
    pub remasters: Vec<i64>,
    #[serde(default, deserialize_with = "capped")]
    pub remakes: Vec<i64>,
    #[serde(default, deserialize_with = "capped")]
    pub ports: Vec<i64>,
    #[serde(default, deserialize_with = "capped")]
    pub alternative_names: Vec<AlternativeName>,
}

#[derive(Debug, Deserialize)]
pub struct AlternativeName {
    pub name: String,
    pub comment: Option<String>,
}

impl GameData {
    /// Rows for `game_links`: (IGDB id, kind).
    pub fn links(&self) -> Vec<(i64, &'static str)> {
        let mut links: Vec<(i64, &'static str)> = self
            .version_parent
            .map(|parent| (parent, "version_of"))
            .into_iter()
            .collect();
        for (ids, kind) in [
            (&self.remasters, "remaster"),
            (&self.remakes, "remake"),
            (&self.ports, "port"),
        ] {
            links.extend(ids.iter().map(|id| (*id, kind)));
        }
        links.retain(|(id, _)| *id != self.id);
        links.sort_unstable();
        links.dedup();
        links
    }
}

/// A list of at most [`LIST_MAX`] items; the rest are read and dropped.
fn capped<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Answer::<T, LIST_MAX>::deserialize(deserializer).map(|answer| answer.0)
}

/// A list of at most `N` items, such as an answer to a query with `limit N`;
/// the rest are read and dropped.
struct Answer<T, const N: usize>(Vec<T>);

impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for Answer<T, N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(Capped::<T, N>(PhantomData))
    }
}

struct Capped<T, const N: usize>(PhantomData<T>);

impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Capped<T, N> {
    type Value = Answer<T, N>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a list")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while items.len() < N {
            match seq.next_element()? {
                Some(item) => items.push(item),
                None => return Ok(Answer(items)),
            }
        }
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(Answer(items))
    }
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
    #[serde(default, deserialize_with = "capped")]
    platforms: Vec<PlatformName>,
    total_rating_count: Option<i64>,
}

#[derive(Deserialize)]
struct PlatformName {
    id: Option<i64>,
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
        let http = crate::http::client(Duration::from_secs(30));
        Self {
            http,
            credentials,
            endpoints,
            token: Mutex::new(None),
            next_request: Mutex::new(Instant::now()),
        }
    }

    pub async fn search(&self, text: &str) -> Result<Vec<SearchHit>, IgdbError> {
        const LIMIT: usize = 20;
        let body = format!(
            "search \"{}\"; fields name,first_release_date,platforms.name,platforms.abbreviation,total_rating_count; limit {LIMIT};",
            escape(text)
        );
        let Answer(mut hits) = self.post::<Answer<RawHit, LIMIT>>("games", &body).await?;
        // IGDB ranks fan hacks next to the real game; rated games come first.
        hits.sort_by_key(|hit| std::cmp::Reverse(hit.total_rating_count.unwrap_or(0)));
        Ok(search_hits(hits))
    }

    /// The games IGDB lists inside a bundle, by name.
    pub async fn bundle_contents(&self, bundle: i64) -> Result<Vec<SearchHit>, IgdbError> {
        const LIMIT: usize = 100;
        let body = format!(
            "fields name,first_release_date,platforms.name,platforms.abbreviation,total_rating_count; where bundles = ({bundle}); sort name asc; limit {LIMIT};"
        );
        let Answer(hits) = self.post::<Answer<RawHit, LIMIT>>("games", &body).await?;
        Ok(search_hits(hits))
    }
}

fn search_hits(hits: Vec<RawHit>) -> Vec<SearchHit> {
    hits.into_iter()
        .map(|hit| SearchHit {
            igdb_id: hit.id,
            name: hit.name,
            year: hit.first_release_date.and_then(year),
            platform_ids: hit.platforms.iter().filter_map(|p| p.id).collect(),
            platforms: hit
                .platforms
                .into_iter()
                .map(|p| p.abbreviation.unwrap_or(p.name))
                .collect(),
            game_id: None,
        })
        .collect()
}

impl Igdb {
    pub async fn games(&self, ids: &[i64]) -> Result<Vec<GameData>, IgdbError> {
        const LIMIT: usize = 500;
        let ids: Vec<String> = ids.iter().map(i64::to_string).collect();
        let body = format!(
            "fields {GAME_FIELDS}; where id = ({}); limit {LIMIT};",
            ids.join(",")
        );
        let Answer(games) = self.post::<Answer<GameData, LIMIT>>("games", &body).await?;
        Ok(games)
    }

    /// Games named exactly `name`, ignoring case.
    pub async fn named(&self, name: &str) -> Result<Vec<GameData>, IgdbError> {
        const LIMIT: usize = 50;
        let body = format!(
            "fields {GAME_FIELDS}; where name ~ \"{}\"; limit {LIMIT};",
            escape(name)
        );
        let Answer(games) = self.post::<Answer<GameData, LIMIT>>("games", &body).await?;
        let name = name.to_lowercase();
        Ok(games
            .into_iter()
            .filter(|g| g.name.to_lowercase() == name)
            .collect())
    }

    pub async fn cover(&self, image_id: &str) -> Result<Vec<u8>, IgdbError> {
        let url = format!("{}/t_cover_big_2x/{image_id}.jpg", self.endpoints.image_url);
        let response = self.http.get(url).send().await?.error_for_status()?;
        crate::http::body(response, COVER_MAX)
            .await?
            .ok_or(IgdbError::TooLarge)
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
                return json(response).await;
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
        let response = self
            .http
            .post(&self.endpoints.token_url)
            .form(&[
                ("client_id", self.credentials.client_id.as_str()),
                ("client_secret", self.credentials.client_secret.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await?
            .error_for_status()?;
        let response: TokenResponse = json(response).await?;
        let lifetime = Duration::from_secs(
            response
                .expires_in
                .saturating_sub(60)
                .min(TOKEN_LIFETIME_MAX),
        );
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

/// Quotes and backslashes escaped; control characters, which could end
/// the query line, dropped.
async fn json<T: DeserializeOwned>(response: reqwest::Response) -> Result<T, IgdbError> {
    let bytes = crate::http::body(response, ANSWER_MAX)
        .await?
        .ok_or(IgdbError::TooLarge)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn escape(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
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
        assert_eq!(escape("Zelda\n; where id = 1;\r\0"), "Zelda; where id = 1;");
    }

    #[test]
    fn long_lists_about_a_game_are_cut_and_links_counted_once() {
        let ports: Vec<i64> = (0..100_000).map(|n| n % 3).collect();
        let names: Vec<_> = (0..1000)
            .map(|n| serde_json::json!({ "name": format!("Name {n}") }))
            .collect();
        let game: GameData = serde_json::from_value(serde_json::json!({
            "id": 1,
            "name": "Tetris",
            "ports": ports,
            "remakes": [2, 2],
            "alternative_names": names,
        }))
        .unwrap();
        assert_eq!(game.ports.len(), LIST_MAX);
        assert_eq!(game.alternative_names.len(), LIST_MAX);
        assert_eq!(game.links(), [(0, "port"), (2, "port"), (2, "remake")]);
    }

    #[test]
    fn answers_keep_only_as_many_games_as_asked_for() {
        let games: Vec<_> = (0..10)
            .map(|n| serde_json::json!({ "id": n, "name": "" }))
            .collect();
        let Answer(games) =
            serde_json::from_value::<Answer<GameData, 3>>(serde_json::Value::from(games)).unwrap();
        assert_eq!(games.iter().map(|g| g.id).collect::<Vec<_>>(), [0, 1, 2]);
    }
}
