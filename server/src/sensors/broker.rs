//! Where the MQTT broker is: `MEMORY_CARD_MQTT_*`, or the Supervisor's MQTT service.

use std::time::Duration;

use reqwest::StatusCode;
use serde::Deserialize;

use crate::config::{Config, MqttBroker, Secret};

const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct Broker {
    pub mqtt: MqttBroker,
    /// This app's slug, for links to its panel.
    pub ingress_slug: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    #[error("cannot reach the Supervisor: {0}")]
    Http(#[from] reqwest::Error),
    #[error("the Supervisor answered {0}")]
    Status(StatusCode),
}

#[derive(Deserialize)]
struct Reply<T> {
    data: T,
}

#[derive(Deserialize)]
struct MqttService {
    host: String,
    port: u16,
    #[serde(default)]
    ssl: bool,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Deserialize)]
struct SelfInfo {
    slug: String,
}

/// `Ok(None)` outside Home Assistant, or when no broker app provides MQTT.
pub async fn find(config: &Config) -> Result<Option<Broker>, BrokerError> {
    if let Some(mqtt) = &config.mqtt {
        return Ok(Some(Broker {
            mqtt: mqtt.clone(),
            ingress_slug: None,
        }));
    }
    let Some(token) = &config.supervisor_token else {
        return Ok(None);
    };
    let http = crate::http::client(TIMEOUT);
    let get = |path: &str| {
        http.get(format!("{}{path}", config.supervisor_url))
            .bearer_auth(token.expose())
    };

    let response = get("/services/mqtt").send().await?;
    match response.status() {
        status if status.is_success() => {}
        // The Supervisor answers 400 while no app provides the service.
        status if status.is_client_error() => return Ok(None),
        status => return Err(BrokerError::Status(status)),
    }
    let service = response.json::<Reply<MqttService>>().await?.data;
    if service.ssl {
        tracing::warn!("the MQTT broker requires TLS, which Memory Card does not support");
        return Ok(None);
    }

    let ingress_slug = match get("/addons/self/info").send().await {
        Ok(response) if response.status().is_success() => response
            .json::<Reply<SelfInfo>>()
            .await
            .ok()
            .map(|info| info.data.slug),
        _ => None,
    };

    Ok(Some(Broker {
        mqtt: MqttBroker {
            host: service.host,
            port: service.port,
            username: service.username.filter(|name| !name.is_empty()),
            password: service.password.filter(|p| !p.is_empty()).map(Secret::new),
        },
        ingress_slug,
    }))
}
