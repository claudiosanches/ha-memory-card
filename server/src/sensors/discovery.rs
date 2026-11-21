//! The MQTT discovery message that creates the sensors, and the state they read.

use serde_json::{Map, Value, json};

use crate::db::sensors::Snapshot;

pub const DISCOVERY_TOPIC: &str = "homeassistant/device/memory_card/config";
/// Home Assistant publishes `online` here when it starts.
pub const HA_STATUS_TOPIC: &str = "homeassistant/status";
pub const STATE_TOPIC: &str = "memory_card/state";
pub const AVAILABILITY_TOPIC: &str = "memory_card/availability";

/// Home Assistant drops longer states.
const MAX_STATE: usize = 255;

struct Sensor {
    key: &'static str,
    name: &'static str,
    icon: &'static str,
    /// Counts get a unit and long-term statistics.
    unit: Option<&'static str>,
    attributes: bool,
}

const SENSORS: [Sensor; 7] = [
    Sensor {
        key: "games",
        name: "Games",
        icon: "mdi:gamepad-variant",
        unit: Some("games"),
        attributes: false,
    },
    Sensor {
        key: "copies",
        name: "Copies",
        icon: "mdi:disc",
        unit: Some("copies"),
        attributes: false,
    },
    Sensor {
        key: "wishlist",
        name: "Wishlist",
        icon: "mdi:heart",
        unit: Some("copies"),
        attributes: false,
    },
    Sensor {
        key: "backlog",
        name: "Backlog",
        icon: "mdi:bookshelf",
        unit: Some("games"),
        attributes: false,
    },
    Sensor {
        key: "playing",
        name: "Playing",
        icon: "mdi:controller",
        unit: Some("games"),
        attributes: true,
    },
    Sensor {
        key: "last_played",
        name: "Last played",
        icon: "mdi:history",
        unit: None,
        attributes: true,
    },
    Sensor {
        key: "last_added",
        name: "Last added",
        icon: "mdi:new-box",
        unit: None,
        attributes: true,
    },
];

/// One device with every sensor. `ingress_slug` links the device page to the app.
pub fn device_config(ingress_slug: Option<&str>) -> Value {
    let version = env!("CARGO_PKG_VERSION");
    let mut device = json!({
        "identifiers": ["memory_card"],
        "name": "Memory Card",
        "model": "Video game collection",
        "sw_version": version,
    });
    if let Some(slug) = ingress_slug {
        device["configuration_url"] = json!(format!("homeassistant://hassio/ingress/{slug}"));
    }

    let components: Map<String, Value> = SENSORS
        .iter()
        .map(|sensor| (sensor.key.to_owned(), component(sensor)))
        .collect();

    json!({
        "device": device,
        "origin": {
            "name": "Memory Card",
            "sw_version": version,
            "support_url": env!("CARGO_PKG_REPOSITORY"),
        },
        "qos": 1,
        "components": components,
    })
}

fn component(sensor: &Sensor) -> Value {
    let mut config = json!({
        "platform": "sensor",
        "name": sensor.name,
        "unique_id": format!("memory_card_{}", sensor.key),
        "default_entity_id": format!("sensor.memory_card_{}", sensor.key),
        "icon": sensor.icon,
        "state_topic": STATE_TOPIC,
        "value_template": format!("{{{{ value_json.{} }}}}", sensor.key),
        "availability_topic": AVAILABILITY_TOPIC,
    });
    if let Some(unit) = sensor.unit {
        config["unit_of_measurement"] = json!(unit);
        config["state_class"] = json!("measurement");
    }
    if sensor.attributes {
        config["json_attributes_topic"] = json!(STATE_TOPIC);
        config["json_attributes_template"] = json!(format!(
            "{{{{ value_json.attributes.{} | tojson }}}}",
            sensor.key
        ));
    }
    config
}

/// The message on [`STATE_TOPIC`]. Missing titles are `null`, which Home Assistant shows as unknown.
pub fn state(snapshot: &Snapshot) -> Value {
    let last_played = snapshot.last_played.as_ref();
    let last_added = snapshot.last_added.as_ref();
    json!({
        "games": snapshot.games,
        "copies": snapshot.copies,
        "wishlist": snapshot.wishlist,
        "backlog": snapshot.backlog,
        "playing": snapshot.playing_count,
        "last_played": last_played.map(|game| fit(&game.title)),
        "last_added": last_added.map(|game| fit(&game.title)),
        "attributes": {
            "playing": { "games": snapshot.playing },
            "last_played": last_played.map_or_else(
                || json!({}),
                |game| json!({ "date": game.date, "platform": game.platform }),
            ),
            "last_added": last_added.map_or_else(
                || json!({}),
                |game| json!({ "added_at": game.added_at, "platform": game.platform }),
            ),
        },
    })
}

fn fit(title: &str) -> String {
    title.chars().take(MAX_STATE).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::sensors::{LastAdded, LastPlayed};

    #[test]
    fn every_sensor_reads_the_state_topic_under_its_own_id() {
        let config = device_config(Some("abc123_memory_card"));
        assert_eq!(
            config["device"]["configuration_url"],
            "homeassistant://hassio/ingress/abc123_memory_card"
        );
        assert_eq!(config["origin"]["name"], "Memory Card");

        let components = config["components"].as_object().unwrap();
        assert_eq!(components.len(), SENSORS.len());
        let games = &components["games"];
        assert_eq!(games["platform"], "sensor");
        assert_eq!(games["unique_id"], "memory_card_games");
        assert_eq!(games["default_entity_id"], "sensor.memory_card_games");
        assert_eq!(games["value_template"], "{{ value_json.games }}");
        assert_eq!(games["state_class"], "measurement");
        assert!(games.get("json_attributes_topic").is_none());

        let last_played = &components["last_played"];
        assert!(last_played.get("unit_of_measurement").is_none());
        assert_eq!(
            last_played["json_attributes_template"],
            "{{ value_json.attributes.last_played | tojson }}"
        );
        assert!(
            device_config(None)["device"]
                .get("configuration_url")
                .is_none()
        );
    }

    #[test]
    fn state_carries_counts_titles_and_attributes() {
        let snapshot = Snapshot {
            games: 3,
            copies: 4,
            wishlist: 1,
            backlog: 2,
            playing: vec!["Celeste".into()],
            playing_count: 1,
            last_played: Some(LastPlayed {
                title: "x".repeat(300),
                date: "2026-11-20".into(),
                platform: None,
            }),
            last_added: None,
        };
        let state = state(&snapshot);
        assert_eq!(state["games"], 3);
        assert_eq!(state["playing"], 1);
        assert_eq!(state["attributes"]["playing"]["games"], json!(["Celeste"]));
        assert_eq!(state["last_played"].as_str().unwrap().len(), MAX_STATE);
        assert_eq!(state["attributes"]["last_played"]["date"], "2026-11-20");
        assert!(state["last_added"].is_null());
        assert_eq!(state["attributes"]["last_added"], json!({}));

        let added = Snapshot {
            last_added: Some(LastAdded {
                title: "Hades".into(),
                added_at: "2026-11-21T10:00:00.000Z".into(),
                platform: "PC".into(),
            }),
            ..snapshot
        };
        assert_eq!(
            super::state(&added)["attributes"]["last_added"]["platform"],
            "PC"
        );
    }
}
