//! Where the MQTT broker is: `MEMORY_CARD_MQTT_*`, or the Supervisor's MQTT service.

use reqwest::Method;
use serde::Deserialize;

use super::supervisor::{self, Reply, SupervisorError};
use crate::config::{Config, MqttBroker, Secret};

#[derive(Debug, Clone)]
pub struct Broker {
    pub mqtt: MqttBroker,
    /// This app's slug, for links to its panel.
    pub ingress_slug: Option<String>,
}

pub type BrokerError = SupervisorError;

#[derive(Deserialize)]
struct MqttService {
    host: String,
    port: u16,
    #[serde(default)]
    ssl: bool,
    username: Option<String>,
    password: Option<String>,
}

/// `Ok(None)` outside Home Assistant, or when no broker app provides MQTT.
pub async fn find(config: &Config, http: &reqwest::Client) -> Result<Option<Broker>, BrokerError> {
    if let Some(mqtt) = &config.mqtt {
        return Ok(Some(Broker {
            mqtt: mqtt.clone(),
            ingress_slug: None,
        }));
    }
    let Some(token) = &config.supervisor_token else {
        return Ok(None);
    };
    let response = supervisor::request(config, http, token, Method::GET, "/services/mqtt")
        .send()
        .await?;
    match response.status() {
        status if status.is_success() => {}
        // The Supervisor answers 400 while no app provides the service.
        status if status.is_client_error() => return Ok(None),
        status => return Err(SupervisorError::Status(status)),
    }
    let service = response.json::<Reply<MqttService>>().await?.data;
    if service.ssl {
        tracing::warn!("the MQTT broker requires TLS, which Memory Card does not support");
        return Ok(None);
    }

    let ingress_slug = supervisor::self_info(config, http)
        .await
        .ok()
        .flatten()
        .map(|info| info.slug);

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
