//! The MQTT discovery message that creates the entities.

use serde_json::{Map, Value, json};

use super::entities::{self, ENTITIES, Entity, PICK_PRESS, Platform};

pub const DISCOVERY_TOPIC: &str = "homeassistant/device/memory_card/config";
pub const STATE_TOPIC: &str = "memory_card/state";
pub const AVAILABILITY_TOPIC: &str = "memory_card/availability";
/// The Roll a pick button publishes here; automations may send filters as JSON.
pub const PICK_TOPIC: &str = "memory_card/pick/roll";

/// One device with every entity. `ingress_slug` links the device page to the app.
pub fn device_config(ingress_slug: Option<&str>) -> Value {
    let version = env!("CARGO_PKG_VERSION");
    let mut device = entities::device(ingress_slug);
    device["identifiers"] = json!(["memory_card"]);

    let components: Map<String, Value> = ENTITIES
        .iter()
        .map(|entity| (entity.key.to_owned(), component(entity)))
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

fn component(entity: &Entity) -> Value {
    let mut config = json!({
        "platform": entity.platform.as_str(),
        "name": entity.name,
        "unique_id": entity.unique_id(),
        "default_entity_id": entity.entity_id(),
        "icon": entity.icon,
        "availability_topic": AVAILABILITY_TOPIC,
    });
    if entity.platform == Platform::Button {
        config["command_topic"] = json!(PICK_TOPIC);
        config["payload_press"] = json!(PICK_PRESS);
        return config;
    }
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
        let config = device_config(Some("abc123_memory_card"));
        assert_eq!(
            config["device"]["configuration_url"],
            "homeassistant://hassio/ingress/abc123_memory_card"
        );
        assert_eq!(config["device"]["identifiers"], json!(["memory_card"]));
        assert_eq!(config["origin"]["name"], "Memory Card");

        let components = config["components"].as_object().unwrap();
        assert_eq!(components.len(), ENTITIES.len());
        let roll = &components["roll_pick"];
        assert_eq!(roll["platform"], "button");
        assert_eq!(roll["command_topic"], PICK_TOPIC);
        assert_eq!(roll["payload_press"], "roll");
        assert_eq!(roll["unique_id"], "memory_card_roll_pick");
        assert_eq!(roll["default_entity_id"], "button.memory_card_roll_pick");
        assert!(roll.get("state_topic").is_none());

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
}
