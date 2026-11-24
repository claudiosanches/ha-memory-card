//! Home Assistant tag scans. Linking waits for the next scan; scanning a
//! linked tag logs a play session for its copy and sets the game to playing.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::{Date, OffsetDateTime};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use utoipa::ToSchema;

use crate::AppState;
use crate::access::{self, Policy};
use crate::config::HomeAssistantApi;
use crate::db;

const RETRY: Duration = Duration::from_secs(15);
const STOP_TIMEOUT: Duration = Duration::from_millis(500);
/// How long a link waits for a scan.
pub const LINK_WAIT: Duration = Duration::from_secs(60);
/// Longer IDs are not tags Home Assistant made.
const TAG_ID_MAX: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TagsStatus {
    /// Outside Home Assistant, without `MEMORY_CARD_HA_URL`.
    Unavailable,
    Connecting,
    Connected,
    /// The connection failed or dropped; retrying.
    Disconnected,
}

impl TagsStatus {
    pub fn initial(api: Option<&HomeAssistantApi>) -> Self {
        if api.is_some() {
            Self::Connecting
        } else {
            Self::Unavailable
        }
    }
}

/// A copy waiting for the next scan.
#[derive(Debug, Clone)]
pub struct PendingLink {
    pub copy_id: i64,
    pub by: String,
    pub until: OffsetDateTime,
}

pub struct Tags {
    stop: Arc<Notify>,
    task: JoinHandle<()>,
}

impl Tags {
    pub fn start(state: AppState) -> Option<Self> {
        let api = state.config.home_assistant.clone()?;
        let stop = Arc::new(Notify::new());
        let task = tokio::spawn(listen(state, api, stop.clone()));
        Some(Self { stop, task })
    }

    pub async fn stop(self) {
        self.stop.notify_one();
        let _ = tokio::time::timeout(STOP_TIMEOUT, self.task).await;
    }
}

fn set_status(state: &AppState, status: TagsStatus) {
    *state.tag_status.lock().expect("tag status lock") = status;
}

async fn listen(state: AppState, api: HomeAssistantApi, stop: Arc<Notify>) {
    let mut failures = 0u32;
    loop {
        tokio::select! {
            () = stop.notified() => return,
            result = subscribe(&state, &api) => {
                if let Err(err) = result {
                    if failures == 0 {
                        tracing::warn!(error = %err, "cannot listen for Home Assistant tag scans, retrying");
                    } else {
                        tracing::debug!(error = %err, "tag scan connection failed");
                    }
                }
                failures += 1;
                set_status(&state, TagsStatus::Disconnected);
            }
        }
        tokio::select! {
            () = stop.notified() => return,
            () = tokio::time::sleep(RETRY) => {}
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum ListenError {
    #[error(transparent)]
    Socket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Home Assistant refused the token")]
    Auth,
    #[error("Home Assistant closed the connection")]
    Closed,
}

#[derive(Deserialize)]
struct Incoming {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    event: Option<TagEvent>,
}

#[derive(Deserialize)]
struct TagEvent {
    data: TagData,
    #[serde(default)]
    context: Option<Context>,
}

#[derive(Deserialize)]
struct TagData {
    tag_id: String,
    /// The phone or reader that scanned, as a Home Assistant device ID.
    #[serde(default)]
    device_id: Option<String>,
}

#[derive(Deserialize)]
struct Context {
    user_id: Option<String>,
}

/// Authenticates, subscribes to `tag_scanned` and handles events until the connection ends.
async fn subscribe(state: &AppState, api: &HomeAssistantApi) -> Result<(), ListenError> {
    let (mut socket, _) = tokio_tungstenite::connect_async(api.websocket.as_str()).await?;
    let mut subscribed = false;
    while let Some(message) = socket.next().await {
        let text = match message? {
            Message::Text(text) => text,
            Message::Close(_) => break,
            _ => continue,
        };
        let Ok(incoming) = serde_json::from_str::<Incoming>(&text) else {
            continue;
        };
        match incoming.kind.as_str() {
            "auth_required" => {
                let auth = json!({"type": "auth", "access_token": api.token.expose()});
                socket.send(Message::text(auth.to_string())).await?;
            }
            "auth_invalid" => return Err(ListenError::Auth),
            "auth_ok" if !subscribed => {
                let request =
                    json!({"id": 1, "type": "subscribe_events", "event_type": "tag_scanned"});
                socket.send(Message::text(request.to_string())).await?;
                subscribed = true;
                set_status(state, TagsStatus::Connected);
                tracing::info!("listening for Home Assistant tag scans");
            }
            "event" => {
                if let Some(event) = incoming.event {
                    let user = event.context.and_then(|c| c.user_id);
                    let reader = event.data.device_id.as_deref();
                    scanned(state, &event.data.tag_id, user.as_deref(), reader).await;
                }
            }
            _ => {}
        }
    }
    Err(ListenError::Closed)
}

/// What a scan did.
#[derive(Debug, PartialEq, Eq)]
pub enum Scan {
    /// The tag now points at the copy that was waiting.
    Linked { copy_id: i64 },
    /// A session was logged for the tag's copy.
    Played { game_id: i64 },
    /// Nothing waits for a link and no copy has the tag.
    Unknown,
}

/// Handles one scan. Public so tests can scan without Home Assistant.
/// `user` is the Home Assistant user who scanned, `None` for a reader with
/// no user; `reader` is the scanning device, which `tag_readers` may tie to
/// a person.
pub async fn scanned(
    state: &AppState,
    tag_id: &str,
    user: Option<&str>,
    reader: Option<&str>,
) -> Scan {
    let tag_id = tag_id.trim();
    if tag_id.is_empty()
        || tag_id.len() > TAG_ID_MAX
        || user.is_some_and(|id| !access::valid_user_id(id))
    {
        return Scan::Unknown;
    }
    let reader = reader.filter(|id| access::valid_user_id(id));
    let person = user.or_else(|| {
        state
            .config
            .tag_readers
            .iter()
            .find(|tag_reader| Some(tag_reader.device.as_str()) == reader)
            .map(|tag_reader| tag_reader.person.as_str())
    });
    let pending = take_link(state, user);
    let result = match pending {
        Some(link) => db::tags::link(&state.db, tag_id, link.copy_id, &link.by)
            .await
            .map(|_| Scan::Linked {
                copy_id: link.copy_id,
            }),
        None => match db::tags::copy_for(&state.db, tag_id).await {
            Ok(Some(copy_id)) => match person {
                Some(person) if Policy::from_config(&state.config).can_track_progress(person) => {
                    let today = local_date(state).await;
                    db::sessions::start_from_tag(
                        &state.db,
                        copy_id,
                        &today,
                        person,
                        state.config.tag_scan_finished,
                    )
                    .await
                    .map(|game_id| Scan::Played { game_id })
                }
                Some(_) => {
                    tracing::info!("ignored a tag scan by someone who cannot keep play");
                    Ok(Scan::Unknown)
                }
                None => {
                    tracing::info!(
                        reader = reader.unwrap_or("unknown"),
                        "ignored a tag scan with no person; map its reader in tag_readers"
                    );
                    Ok(Scan::Unknown)
                }
            },
            Ok(None) => Ok(Scan::Unknown),
            Err(err) => Err(err.into()),
        },
    };
    match result {
        Ok(scan) => {
            if scan != Scan::Unknown {
                tracing::info!(?scan, "handled a tag scan");
                state.changes.announce();
            }
            scan
        }
        Err(err) => {
            tracing::warn!(error = %err, "cannot handle a tag scan");
            Scan::Unknown
        }
    }
}

/// The link waiting for this scan, if any. A scan by another named user
/// leaves it waiting; a reader with no user may complete it.
fn take_link(state: &AppState, user: Option<&str>) -> Option<PendingLink> {
    let mut waiting = state.tag_link.lock().expect("tag link lock");
    let link = waiting.as_ref()?;
    if link.until <= OffsetDateTime::now_utc() {
        *waiting = None;
        return None;
    }
    if user.is_some_and(|id| id != link.by) {
        return None;
    }
    waiting.take()
}

/// Today in Home Assistant's time zone, asked of Home Assistant; the UTC date if it does not answer.
async fn local_date(state: &AppState) -> String {
    let utc = OffsetDateTime::now_utc().date().to_string();
    let Some(api) = &state.config.home_assistant else {
        return utc;
    };
    let response = state
        .http
        .post(format!("{}/template", api.rest))
        .timeout(Duration::from_secs(5))
        .bearer_auth(api.token.expose())
        .json(&json!({"template": "{{ now().date() }}"}))
        .send()
        .await;
    let text = match response {
        Ok(response) if response.status().is_success() => response.text().await.ok(),
        _ => None,
    };
    text.map(|t| t.trim().to_owned())
        .filter(|t| {
            Date::parse(t, time::macros::format_description!("[year]-[month]-[day]")).is_ok()
        })
        .unwrap_or(utc)
}
