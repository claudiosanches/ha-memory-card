//! The feed the Memory Card integration reads instead of MQTT: the entities,
//! their state after each change, and a roll. Its port is only reachable on
//! the Supervisor's network, and every request carries the token the app
//! hands Home Assistant through Supervisor discovery.

use std::collections::VecDeque;
use std::convert::Infallible;
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Json;
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde_json::json;
use subtle::ConstantTimeEq;
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore, watch};

use super::entities;
use super::{SensorsStatus, broker, set_status, supervisor};
use crate::AppState;
use crate::access::{self, Policy};
use crate::config::Secret;
use crate::db;
use crate::error::ApiError;
use crate::serve::{Limits, serve};

/// The Home Assistant user behind a roll; absent for automations.
pub const USER_HEADER: &str = "x-home-assistant-user";
/// Bumped when the integration could misread a message.
pub const API_VERSION: u32 = 1;
pub const FEEDS_MAX: usize = 4;
/// Home Assistant needs a feed and a roll at a time; the rest is room for retries.
const CONNECTIONS_MAX: usize = 32;
/// Longer than aiohttp's 15 s keep-alive, so Home Assistant never reuses a
/// connection the app is closing.
const HEADER_TIMEOUT: Duration = Duration::from_secs(20);
const PING: Duration = Duration::from_secs(30);
const BODY_MAX: usize = 4 * 1024;
const RETRY: Duration = Duration::from_secs(15);
const STOP_TIMEOUT: Duration = Duration::from_millis(500);

/// The token file holds 32 random bytes as hex.
const TOKEN_LEN: usize = 64;

/// Reads the token, or creates one readable only by the app.
pub fn token(path: &Path) -> std::io::Result<Secret> {
    match std::fs::read_to_string(path) {
        Ok(saved) if valid_token(saved.trim()) => {
            // A restored or copied file may have come back readable by others.
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
            return Ok(Secret::new(saved.trim()));
        }
        Ok(_) => std::fs::remove_file(path)?,
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    let mut bytes = [0u8; TOKEN_LEN / 2];
    getrandom::fill(&mut bytes).map_err(std::io::Error::other)?;
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(token.as_bytes())?;
    file.sync_all()?;
    Ok(Secret::new(token))
}

fn valid_token(token: &str) -> bool {
    token.len() == TOKEN_LEN && token.bytes().all(|b| b.is_ascii_hexdigit())
}

/// What the feed's handlers share.
pub struct Feed {
    state: AppState,
    token: Secret,
    /// What open feeds get next: a state line, after a hello line when the
    /// entities changed. `None` until the first is read.
    latest: watch::Sender<Option<Arc<str>>>,
    /// The current hello and state lines, which a new feed starts with.
    current: Mutex<(String, Option<String>)>,
    closed: watch::Sender<bool>,
    places: Arc<Semaphore>,
    /// How many feeds are open, for the status.
    open: Mutex<usize>,
}

impl Feed {
    pub fn new(state: AppState, token: Secret) -> Arc<Self> {
        Arc::new(Self {
            state,
            token,
            latest: watch::Sender::new(None),
            current: Mutex::new((hello(&db::sensors::Snapshot::default()), None)),
            closed: watch::Sender::new(false),
            places: Arc::new(Semaphore::new(FEEDS_MAX)),
            open: Mutex::new(0),
        })
    }

    /// Reads the sensors and hands them to every open feed, unless unchanged.
    /// The entities follow who keeps play, so a change to them goes out first.
    pub async fn refresh(&self) {
        let snapshot = match db::sensors::snapshot(&self.state.db).await {
            Ok(snapshot) => snapshot,
            Err(err) => {
                tracing::warn!(error = %err, "cannot read sensor values");
                return;
            }
        };
        let hello = hello(&snapshot);
        let state = json!({ "type": "state", "state": entities::state(&snapshot) }).to_string();
        let message = {
            let mut current = self.current.lock().expect("feed lines lock");
            let entities_changed = current.0 != hello;
            if !entities_changed && current.1.as_deref() == Some(state.as_str()) {
                return;
            }
            let message = if entities_changed {
                format!("{hello}\n{state}")
            } else {
                state.clone()
            };
            *current = (hello, Some(state));
            message
        };
        self.latest.send_replace(Some(message.into()));
    }

    /// Ends every open feed.
    pub fn close(&self) {
        self.closed.send_replace(true);
    }

    fn opened(&self) {
        let mut open = self.open.lock().expect("feed count lock");
        *open += 1;
        set_status(&self.state, SensorsStatus::Connected);
    }

    fn ended(&self) {
        let mut open = self.open.lock().expect("feed count lock");
        *open -= 1;
        if *open == 0 {
            set_status(&self.state, SensorsStatus::Waiting);
        }
    }
}

pub fn router(feed: Arc<Feed>) -> Router {
    Router::new()
        .route("/integration/v1/feed", get(stream))
        .route("/integration/v1/roll", post(roll))
        .layer(DefaultBodyLimit::max(BODY_MAX))
        .layer(middleware::from_fn_with_state(feed.clone(), authorize))
        .with_state(feed)
}

async fn authorize(State(feed): State<Arc<Feed>>, request: Request, next: Next) -> Response {
    let sent = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();
    if bool::from(sent.as_bytes().ct_eq(feed.token.expose().as_bytes())) {
        next.run(request).await
    } else {
        // Closing keeps a refused client from holding its connection open.
        (
            StatusCode::UNAUTHORIZED,
            [(header::CONNECTION, "close")],
            Json(json!({ "code": "unauthorized" })),
        )
            .into_response()
    }
}

/// One open feed; ending it frees its place.
struct Open {
    feed: Arc<Feed>,
    _place: OwnedSemaphorePermit,
}

impl Drop for Open {
    fn drop(&mut self) {
        self.feed.ended();
    }
}

struct Lines {
    pending: VecDeque<Bytes>,
    latest: watch::Receiver<Option<Arc<str>>>,
    closed: watch::Receiver<bool>,
    ping: tokio::time::Interval,
    _open: Open,
}

impl Lines {
    /// `None` once the app stops.
    async fn next(&mut self) -> Option<Bytes> {
        if *self.closed.borrow() {
            return None;
        }
        if let Some(next) = self.pending.pop_front() {
            return Some(next);
        }
        loop {
            tokio::select! {
                changed = self.latest.changed() => {
                    changed.ok()?;
                    if let Some(state) = self.latest.borrow_and_update().as_deref() {
                        return Some(Bytes::from(format!("{state}\n")));
                    }
                }
                _ = self.ping.tick() => return Some(line(&json!({ "type": "ping" }))),
                _ = self.closed.wait_for(|closed| *closed) => return None,
            }
        }
    }
}

/// The hello line: the app, its device and the entities `snapshot` calls for.
fn hello(snapshot: &db::sensors::Snapshot) -> String {
    json!({
        "type": "hello",
        "api": API_VERSION,
        "version": env!("CARGO_PKG_VERSION"),
        "device": entities::device(None),
        "entities": entities::entities(snapshot),
    })
    .to_string()
}

fn line(value: &serde_json::Value) -> Bytes {
    let mut line = value.to_string();
    line.push('\n');
    Bytes::from(line)
}

/// Newline-delimited JSON: `hello` with the entities, then `state` now and
/// after each change, and `ping` while nothing changes.
async fn stream(State(feed): State<Arc<Feed>>) -> Result<Response, ApiError> {
    let place = feed
        .places
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError::Busy("too_many_feeds"))?;
    feed.opened();
    let open = Open {
        feed: feed.clone(),
        _place: place,
    };

    let mut latest = feed.latest.subscribe();
    latest.mark_unchanged();
    let (hello, first) = feed.current.lock().expect("feed lines lock").clone();
    let hello = Bytes::from(format!("{hello}\n"));
    let first = first.map(|state| Bytes::from(format!("{state}\n")));
    let lines = Lines {
        pending: std::iter::once(hello).chain(first).collect(),
        latest,
        closed: feed.closed.subscribe(),
        ping: tokio::time::interval_at(tokio::time::Instant::now() + PING, PING),
        _open: open,
    };
    let lines = futures_util::stream::unfold(lines, |mut lines| async move {
        let next = lines.next().await?;
        Some((Ok::<_, Infallible>(next), lines))
    });
    Ok((
        [
            (header::CONTENT_TYPE, "application/x-ndjson"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        Body::from_stream(lines),
    )
        .into_response())
}

/// Picks a game from one person's backlog, as Pick for me does, with optional
/// filters. The person is the Home Assistant user in [`USER_HEADER`], who
/// rolls only for themselves, or the `person` an automation names.
async fn roll(
    State(feed): State<Arc<Feed>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    let user = match headers.get(USER_HEADER) {
        None => None,
        Some(value) => Some(
            value
                .to_str()
                .ok()
                .filter(|id| access::valid_user_id(id))
                .ok_or(ApiError::Invalid {
                    code: "invalid_user",
                    field: None,
                })?,
        ),
    };
    let request = entities::roll_request(&body).map_err(|_| ApiError::Invalid {
        code: "invalid_filters",
        field: None,
    })?;
    let person = match (user, request.person.as_deref()) {
        (Some(user), None) => user,
        (Some(user), Some(named)) if named == user => user,
        (Some(_), Some(_)) => return Err(ApiError::Forbidden("not_your_roll")),
        (None, Some(named)) if access::valid_user_id(named) => {
            if !super::may_roll_for(&feed.state, named).await? {
                return Err(ApiError::Forbidden("play_not_allowed"));
            }
            named
        }
        (None, Some(_)) => {
            return Err(ApiError::Invalid {
                code: "invalid_user",
                field: Some("person"),
            });
        }
        (None, None) => {
            return Err(ApiError::Invalid {
                code: "person_required",
                field: Some("person"),
            });
        }
    };
    if !Policy::from_config(&feed.state.config).can_track_progress(person) {
        return Err(ApiError::Forbidden("play_not_allowed"));
    }
    let game = db::suggestions::suggest(&feed.state.db, person, request.filter)
        .await?
        .game;
    feed.state.changes.announce();
    let platforms = db::sensors::owned_platforms(&feed.state.db, game.id).await?;
    Ok(Json(json!({ "title": game.title, "platforms": platforms })).into_response())
}

/// Serves the feed until `stop`, after clearing what MQTT published and
/// telling Home Assistant where to find it.
pub async fn run(state: &AppState, stop: &Notify) {
    let token = match token(&state.config.integration_token_path()) {
        Ok(token) => token,
        Err(err) => {
            tracing::error!(error = %err, "cannot read or create the integration token");
            set_status(state, SensorsStatus::Failed);
            return;
        }
    };
    let listener = match tokio::net::TcpListener::bind(state.config.integration_listen).await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::error!(error = %err, "cannot listen for the integration");
            set_status(state, SensorsStatus::Failed);
            return;
        }
    };
    let port = listener.local_addr().map_or(0, |addr| addr.port());
    tracing::info!(port, "serving the Memory Card integration");

    let feed = Feed::new(state.clone(), token.clone());
    let mut closed = feed.closed.subscribe();
    let limits = Limits {
        connections: CONNECTIONS_MAX,
        header_timeout: HEADER_TIMEOUT,
        // Other apps on the Supervisor's network could otherwise take every place.
        peers: Some(state.config.integration_peers.clone().into()),
    };
    let server = tokio::spawn(serve(listener, router(feed.clone()), limits, async move {
        let _ = closed.wait_for(|closed| *closed).await;
    }));
    let handover = tokio::spawn(hand_over(state.clone(), token, port));

    feed.refresh().await;
    loop {
        tokio::select! {
            () = stop.notified() => break,
            () = state.changes.next_for_sensors() => {
                tokio::time::sleep(super::DEBOUNCE).await;
                feed.refresh().await;
            }
        }
    }

    feed.close();
    handover.abort();
    if tokio::time::timeout(STOP_TIMEOUT, server).await.is_err() {
        tracing::debug!("the integration feed did not stop in time");
    }
}

/// Removes the MQTT entities so the integration's take their IDs, then
/// announces the feed, retrying until the Supervisor takes it.
async fn hand_over(state: AppState, token: Secret, port: u16) {
    if let Ok(Some(broker)) = broker::find(&state.config, &state.http).await {
        super::remove(&broker.mqtt).await;
    }
    let Some(supervisor_token) = state.config.supervisor_token.clone() else {
        tracing::info!("outside Home Assistant, nothing announces the integration feed");
        return;
    };
    let mut failures = 0u32;
    loop {
        match announce(&state, &supervisor_token, &token, port).await {
            Ok(()) => return,
            Err(err) => {
                if failures == 0 {
                    tracing::warn!(error = %err, "cannot announce the integration feed, retrying");
                } else {
                    tracing::debug!(error = %err, "cannot announce the integration feed");
                }
                failures += 1;
                tokio::time::sleep(RETRY).await;
            }
        }
    }
}

async fn announce(
    state: &AppState,
    supervisor_token: &Secret,
    token: &Secret,
    port: u16,
) -> Result<(), supervisor::SupervisorError> {
    let Some(info) = supervisor::self_info(&state.config, &state.http).await? else {
        return Ok(());
    };
    let config = json!({ "host": info.hostname, "port": port, "token": token.expose() });
    let uuid = supervisor::announce(&state.config, &state.http, supervisor_token, config).await?;
    if let Err(err) = std::fs::write(state.config.integration_discovery_path(), &uuid) {
        tracing::warn!(error = %err, "cannot remember the discovery message");
    }
    tracing::info!("announced the integration feed to Home Assistant");
    Ok(())
}

/// Takes back an earlier announcement, so Home Assistant removes the
/// integration and its entities. Returns whether there was one.
pub async fn withdraw(state: &AppState) -> bool {
    let path = state.config.integration_discovery_path();
    let uuid = match std::fs::read_to_string(&path) {
        Ok(uuid) => uuid.trim().to_owned(),
        Err(err) if err.kind() == ErrorKind::NotFound => return false,
        Err(err) => {
            tracing::warn!(error = %err, "cannot read the discovery message id");
            return false;
        }
    };
    let Some(supervisor_token) = &state.config.supervisor_token else {
        return false;
    };
    match supervisor::withdraw(&state.config, &state.http, supervisor_token, &uuid).await {
        Ok(()) => {
            let _ = std::fs::remove_file(&path);
            tracing::info!("removed the Memory Card integration from Home Assistant");
            true
        }
        Err(err) => {
            tracing::warn!(error = %err, "cannot remove the integration's discovery message");
            false
        }
    }
}
