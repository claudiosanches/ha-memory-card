//! Home Assistant sensors, created through MQTT discovery or read by the
//! Memory Card integration, and refreshed after each change.

pub mod broker;
pub mod discovery;
pub mod entities;
pub mod integration;
pub mod supervisor;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use rumqttc::{AsyncClient, Event, EventLoop, LastWill, MqttOptions, Outgoing, Packet, QoS};
use serde::Serialize;
use tokio::sync::{Notify, mpsc};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use utoipa::ToSchema;

use self::broker::Broker;
use self::discovery::{AVAILABILITY_TOPIC, DISCOVERY_TOPIC, PICK_TOPIC, STATE_TOPIC};
use crate::AppState;
use crate::changes::Changes;
use crate::config::{Config, MqttBroker, SensorsMode};
use crate::db;

const CLIENT_ID: &str = "memory_card";
/// Changes arriving within this window go out as one update.
const DEBOUNCE: Duration = Duration::from_secs(1);
/// After removing the integration, Home Assistant needs a moment to free its
/// entity IDs for the MQTT ones.
const HANDOVER: Duration = Duration::from_secs(5);
const RETRY: Duration = Duration::from_secs(15);
/// MQTT names no one, so presses closer together than this are dropped.
const ROLL_GAP: Duration = Duration::from_secs(1);
const STOP_TIMEOUT: Duration = Duration::from_millis(500);
const REMOVE_TIMEOUT: Duration = Duration::from_secs(10);
/// How long the broker gets to hand back a device it kept.
const LOOK_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SensorsStatus {
    /// Turned off in the app options.
    Off,
    /// Outside Home Assistant, or no broker app is installed.
    NoBroker,
    Connecting,
    Connected,
    /// The connection failed or dropped; retrying.
    Disconnected,
    /// Ready for the Memory Card integration, which has not connected.
    Waiting,
    /// The integration feed could not start; the log says why.
    Failed,
}

impl SensorsStatus {
    pub fn initial(config: &Config) -> Self {
        if config.sensors == SensorsMode::Off {
            Self::Off
        } else if config.sensors == SensorsMode::Integration {
            Self::Waiting
        } else if config.mqtt.is_none() && config.supervisor_token.is_none() {
            Self::NoBroker
        } else {
            Self::Connecting
        }
    }
}

/// Wakes the publisher after any successful write.
pub async fn track_changes(
    State(changes): State<Changes>,
    request: Request,
    next: Next,
) -> Response {
    let writes = !matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let response = next.run(request).await;
    if writes && response.status().is_success() {
        changes.announce();
    }
    response
}

pub struct Sensors {
    stop: Arc<Notify>,
    task: JoinHandle<()>,
}

impl Sensors {
    pub fn start(state: AppState) -> Self {
        let stop = Arc::new(Notify::new());
        let task = tokio::spawn(run(state, stop.clone()));
        Self { stop, task }
    }

    /// Marks the sensors unavailable before the app exits.
    pub async fn stop(self) {
        self.stop.notify_one();
        if tokio::time::timeout(STOP_TIMEOUT, self.task).await.is_err() {
            tracing::debug!("sensors did not stop in time");
        }
    }
}

enum Signal {
    Connected,
    /// Someone pressed Roll a pick, or an automation sent filters.
    Roll(Vec<u8>),
    /// The broker still holds a device an earlier run published.
    Published,
    Lost,
    /// Our own disconnect went out.
    Closed,
}

fn set_status(state: &AppState, status: SensorsStatus) {
    *state.sensors.lock().expect("sensors status lock") = status;
}

async fn run(state: AppState, stop: Arc<Notify>) {
    if state.config.sensors == SensorsMode::Integration {
        integration::run(&state, &stop).await;
        return;
    }
    let handed_over = integration::withdraw(&state).await;
    let broker = loop {
        tokio::select! {
            () = stop.notified() => return,
            found = broker::find(&state.config, &state.http) => match found {
                Ok(found) => break found,
                Err(err) => {
                    tracing::warn!(error = %err, "cannot look up the MQTT broker");
                    tokio::select! {
                        () = stop.notified() => return,
                        () = tokio::time::sleep(RETRY) => {}
                    }
                }
            },
        }
    };
    let Some(broker) = broker else {
        if state.config.sensors == SensorsMode::Mqtt {
            tracing::info!("no MQTT broker, Home Assistant sensors are off");
            set_status(&state, SensorsStatus::NoBroker);
        }
        return;
    };
    match state.config.sensors {
        SensorsMode::Mqtt => {
            if handed_over {
                tokio::select! {
                    () = stop.notified() => return,
                    () = tokio::time::sleep(HANDOVER) => {}
                }
            }
            publish(&state, &stop, broker).await;
        }
        SensorsMode::Off => remove(&broker.mqtt).await,
        SensorsMode::Integration => {}
    }
}

fn options(broker: &MqttBroker, will: bool) -> MqttOptions {
    let mut options = MqttOptions::new(CLIENT_ID, &broker.host, broker.port);
    options.set_keep_alive(Duration::from_secs(30));
    // The playing list can outgrow the 10 KiB default.
    options.set_max_packet_size(64 * 1024, 256 * 1024);
    if let Some(username) = &broker.username {
        let password = broker.password.as_ref().map_or("", |p| p.expose());
        options.set_credentials(username, password);
    }
    if will {
        options.set_last_will(LastWill::new(
            AVAILABILITY_TOPIC,
            "offline",
            QoS::AtLeastOnce,
            true,
        ));
    }
    options
}

/// Drives the connection, reconnecting after a pause, and reports what matters.
async fn poll(mut eventloop: EventLoop, signals: mpsc::Sender<Signal>) {
    let mut failures = 0u32;
    loop {
        let signal = match eventloop.poll().await {
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                failures = 0;
                Signal::Connected
            }
            // Only a press made now, not one the broker kept.
            Ok(Event::Incoming(Packet::Publish(message)))
                if message.topic == PICK_TOPIC && !message.retain =>
            {
                Signal::Roll(message.payload.to_vec())
            }
            Ok(Event::Incoming(Packet::Publish(message)))
                if message.topic == DISCOVERY_TOPIC && !message.payload.is_empty() =>
            {
                Signal::Published
            }
            Ok(Event::Outgoing(Outgoing::Disconnect)) => {
                let _ = signals.send(Signal::Closed).await;
                return;
            }
            Ok(_) => continue,
            Err(err) => {
                if failures == 0 {
                    tracing::warn!(error = %err, "MQTT connection failed, retrying");
                } else {
                    tracing::debug!(error = %err, "MQTT connection failed");
                }
                failures += 1;
                if signals.send(Signal::Lost).await.is_err() {
                    return;
                }
                tokio::time::sleep(RETRY).await;
                continue;
            }
        };
        // Rolls and repeated notices may be dropped while the publisher is
        // busy; waiting for it would stop the polling it may be waiting on.
        let delivered = match signal {
            Signal::Connected => signals.send(signal).await.is_ok(),
            signal => match signals.try_send(signal) {
                Err(mpsc::error::TrySendError::Closed(_)) => false,
                Err(mpsc::error::TrySendError::Full(_)) => {
                    tracing::debug!("dropped an MQTT message while busy");
                    true
                }
                Ok(()) => true,
            },
        };
        if !delivered {
            return;
        }
    }
}

async fn publish(state: &AppState, stop: &Notify, broker: Broker) {
    let (client, eventloop) = AsyncClient::new(options(&broker.mqtt, true), 32);
    let (signals, mut events) = mpsc::channel(8);
    let poller = tokio::spawn(poll(eventloop, signals));
    let config = discovery::device_config(broker.ingress_slug.as_deref()).to_string();
    let mut connected = false;
    let mut sent: Option<String> = None;
    let mut last_roll: Option<Instant> = None;

    loop {
        tokio::select! {
            () = stop.notified() => break,
            event = events.recv() => match event {
                Some(Signal::Connected) => {
                    if !connected {
                        tracing::info!(host = %broker.mqtt.host, "publishing Home Assistant sensors");
                    }
                    connected = true;
                    set_status(state, SensorsStatus::Connected);
                    // A message someone left retained here would come back on
                    // every connect; one too large for the client would end
                    // each connection.
                    send(&client, PICK_TOPIC, Vec::new());
                    let _ = client.try_subscribe(PICK_TOPIC, QoS::AtLeastOnce);
                    // Kept by the broker, so Home Assistant reads them again
                    // when it restarts.
                    send(&client, DISCOVERY_TOPIC, config.clone());
                    send(&client, AVAILABILITY_TOPIC, "online");
                    sent = send_state(&client, state, None).await;
                }
                Some(Signal::Roll(_)) if last_roll.is_some_and(|at| at.elapsed() < ROLL_GAP) => {
                    tracing::debug!("ignored a roll right after another");
                }
                Some(Signal::Roll(payload)) => {
                    last_roll = Some(Instant::now());
                    roll(state, &payload).await;
                    if connected {
                        sent = send_state(&client, state, sent).await;
                    }
                }
                // Only `remove` listens for the device.
                Some(Signal::Published) => {}
                Some(Signal::Lost) => {
                    connected = false;
                    set_status(state, SensorsStatus::Disconnected);
                }
                Some(Signal::Closed) | None => break,
            },
            () = state.changes.next_for_sensors() => {
                tokio::time::sleep(DEBOUNCE).await;
                if connected {
                    sent = send_state(&client, state, sent).await;
                }
            }
        }
    }

    if connected {
        send(&client, AVAILABILITY_TOPIC, "offline");
        let _ = client.try_disconnect();
        closed(&mut events).await;
    }
    poller.abort();
}

/// Picks a game the way the app's Roll button does; the sensors show it.
async fn roll(state: &AppState, payload: &[u8]) {
    // MQTT carries no user, so anyone who can press the button rolls once
    // managers are set.
    if !crate::access::Policy::from_config(&state.config).can_roll(None) {
        tracing::info!("ignored a roll from Home Assistant: no managers are set");
        return;
    }
    let filter = match entities::pick_filter(payload) {
        Ok(filter) => filter,
        Err(err) => {
            tracing::warn!(error = %err, "ignored a pick request with unreadable filters");
            return;
        }
    };
    match db::suggestions::suggest(&state.db, filter).await {
        Ok(pick) => tracing::info!(title = %pick.game.title, "rolled a pick for Home Assistant"),
        Err(err) => tracing::info!(error = %err, "no game to pick for Home Assistant"),
    }
}

/// Lets the poller flush the last messages and the disconnect.
async fn closed(events: &mut mpsc::Receiver<Signal>) {
    let _ = tokio::time::timeout(STOP_TIMEOUT, async {
        while let Some(signal) = events.recv().await {
            if matches!(signal, Signal::Closed | Signal::Lost) {
                break;
            }
        }
    })
    .await;
}

/// Queues a message without waiting, since the poller that sends it may be
/// waiting on this task; a full queue drops it. Returns whether it was queued.
fn send(client: &AsyncClient, topic: &str, payload: impl Into<Vec<u8>>) -> bool {
    match client.try_publish(topic, QoS::AtLeastOnce, true, payload.into()) {
        Ok(()) => true,
        Err(err) => {
            tracing::debug!(error = %err, topic, "cannot publish to MQTT");
            false
        }
    }
}

/// Publishes the state unless it matches `sent`, and returns what the broker now holds.
async fn send_state(
    client: &AsyncClient,
    state: &AppState,
    sent: Option<String>,
) -> Option<String> {
    let snapshot = match db::sensors::snapshot(&state.db).await {
        Ok(snapshot) => snapshot,
        Err(err) => {
            tracing::warn!(error = %err, "cannot read sensor values");
            return sent;
        }
    };
    let payload = entities::state(&snapshot).to_string();
    if sent.as_deref() == Some(payload.as_str()) {
        return sent;
    }
    if send(client, STATE_TOPIC, payload.clone()) {
        Some(payload)
    } else {
        sent
    }
}

/// Waits for a signal `wanted` picks, until the connection ends or `wait` passes.
async fn until(
    events: &mut mpsc::Receiver<Signal>,
    wait: Duration,
    wanted: fn(&Signal) -> bool,
) -> bool {
    tokio::time::timeout(wait, async {
        loop {
            match events.recv().await {
                Some(signal) if wanted(&signal) => break true,
                Some(Signal::Lost | Signal::Closed) | None => break false,
                Some(_) => {}
            }
        }
    })
    .await
    .unwrap_or(false)
}

/// Clears what an earlier run published, so turning sensors off removes the
/// device. A broker holding nothing of ours is left alone, since clearing it
/// makes Home Assistant log a warning.
pub(crate) async fn remove(broker: &MqttBroker) {
    let (client, eventloop) = AsyncClient::new(options(broker, false), 8);
    let (signals, mut events) = mpsc::channel(8);
    let poller = tokio::spawn(poll(eventloop, signals));

    if until(&mut events, REMOVE_TIMEOUT, |s| {
        matches!(s, Signal::Connected)
    })
    .await
    {
        let _ = client.try_subscribe(DISCOVERY_TOPIC, QoS::AtLeastOnce);
        // The broker sends a kept message right after the subscription.
        if until(&mut events, LOOK_TIMEOUT, |s| {
            matches!(s, Signal::Published)
        })
        .await
        {
            for topic in [DISCOVERY_TOPIC, STATE_TOPIC, AVAILABILITY_TOPIC] {
                send(&client, topic, Vec::new());
            }
        }
        let _ = client.try_disconnect();
        closed(&mut events).await;
    }
    poller.abort();
}
