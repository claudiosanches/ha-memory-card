//! The MQTT discovery message that creates the entities.

use serde_json::{Map, Value, json};

use super::entities::{self, Entity, Platform};

pub const DISCOVERY_TOPIC: &str = "homeassistant/device/memory_card/config";
pub const STATE_TOPIC: &str = "memory_card/state";
pub const AVAILABILITY_TOPIC: &str = "memory_card/availability";
/// Automations roll here with `{"person": "<user id>"}` and any filters.
pub const PICK_TOPIC: &str = "memory_card/pick/roll";

/// One device with every sensor. The roll button stays with the integration:
/// a press over MQTT names no one. `gone` are keys published before that no
/// longer have an entity, which Home Assistant then removes. `ingress_slug`
/// links the device page to the app.
pub fn device_config(ingress_slug: Option<&str>, entities: &[Entity], gone: &[String]) -> Value {
    let version = env!("CARGO_PKG_VERSION");
    let mut device = entities::device(ingress_slug);
    device["identifiers"] = json!(["memory_card"]);

    let mut components: Map<String, Value> = entities
        .iter()
        .filter(|entity| entity.platform == Platform::Sensor)
        .map(|entity| (entity.key.clone(), component(entity)))
        .collect();
    for key in gone {
        components.insert(key.clone(), json!({ "platform": "sensor" }));
    }

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

fn component(entity: &Entity) -> Value {
    let mut config = json!({
        "platform": entity.platform.as_str(),
        "name": entity.name,
        "unique_id": entity.unique_id(),
        "default_entity_id": entity.entity_id(),
        "icon": entity.icon,
        "availability_topic": AVAILABILITY_TOPIC,
    });
    config["state_topic"] = json!(STATE_TOPIC);
    config["value_template"] = json!(format!("{{{{ value_json.{} }}}}", entity.key));
    if let Some(unit) = entity.unit {
        config["unit_of_measurement"] = json!(unit);
    }
    if let Some(state_class) = entity.state_class {
        config["state_class"] = json!(state_class);
    }
    if entity.attributes {
        config["json_attributes_topic"] = json!(STATE_TOPIC);
        config["json_attributes_template"] = json!(format!(
            "{{{{ value_json.attributes.{} | tojson }}}}",
            entity.key
        ));
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sensor_reads_the_state_topic_under_its_own_id() {
        use crate::db::sensors::{PersonPlay, Snapshot};
        let snapshot = Snapshot {
            people: vec![PersonPlay {
                id: "ana".into(),
                name: Some("Ana".into()),
                backlog: 0,
                playing: Vec::new(),
                playing_count: 0,
                last_played: None,
            }],
            ..Snapshot::default()
        };
        let all = entities::entities(&snapshot);
        let config = device_config(Some("abc123_memory_card"), &all, &[]);
        assert_eq!(
            config["device"]["configuration_url"],
            "homeassistant://hassio/ingress/abc123_memory_card"
        );
        assert_eq!(config["device"]["identifiers"], json!(["memory_card"]));
        assert_eq!(config["origin"]["name"], "Memory Card");

        let components = config["components"].as_object().unwrap();
        assert_eq!(components.len(), all.len() - 1);
        assert!(components.get("roll_pick").is_none());

        let games = &components["games"];
        assert_eq!(games["platform"], "sensor");
        assert_eq!(games["unique_id"], "memory_card_games");
        assert_eq!(games["default_entity_id"], "sensor.memory_card_games");
        assert_eq!(games["value_template"], "{{ value_json.games }}");
        assert_eq!(games["state_class"], "measurement");
        assert!(games.get("json_attributes_topic").is_none());

        let last_played = &components["person_ana_last_played"];
        assert_eq!(last_played["name"], "Ana last played");
        assert_eq!(
            last_played["default_entity_id"],
            "sensor.memory_card_ana_last_played"
        );
        assert!(last_played.get("unit_of_measurement").is_none());
        assert_eq!(
            last_played["json_attributes_template"],
            "{{ value_json.attributes.person_ana_last_played | tojson }}"
        );

        let household = entities::entities(&Snapshot::default());
        let gone = vec!["person_ana_backlog".to_owned()];
        let config = device_config(None, &household, &gone);
        assert_eq!(
            config["components"]["person_ana_backlog"],
            json!({ "platform": "sensor" })
        );
        assert!(config["device"].get("configuration_url").is_none());
    }
}
