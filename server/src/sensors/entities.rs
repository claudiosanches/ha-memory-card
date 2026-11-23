//! The entities Home Assistant gets and the state they read, whether MQTT or
//! the integration carries them.

use serde::Serialize;
use serde_json::{Value, json};

use crate::db::sensors::Snapshot;
use crate::db::suggestions::SuggestionFilter;

/// What the Roll a pick button sends; anything else must be filters as JSON.
pub const PICK_PRESS: &str = "roll";
/// Home Assistant drops longer states.
const MAX_STATE: usize = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Sensor,
    Button,
}

impl Platform {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sensor => "sensor",
            Self::Button => "button",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Entity {
    /// Names its value in the state, and its IDs.
    pub key: &'static str,
    pub platform: Platform,
    pub name: &'static str,
    pub icon: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<&'static str>,
    /// Counts keep long-term statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_class: Option<&'static str>,
    /// The state also carries `attributes.<key>`.
    pub attributes: bool,
}

impl Entity {
    pub fn unique_id(&self) -> String {
        format!("memory_card_{}", self.key)
    }

    pub fn entity_id(&self) -> String {
        format!("{}.memory_card_{}", self.platform.as_str(), self.key)
    }
}

const fn count(
    key: &'static str,
    name: &'static str,
    icon: &'static str,
    unit: &'static str,
) -> Entity {
    Entity {
        key,
        platform: Platform::Sensor,
        name,
        icon,
        unit: Some(unit),
        state_class: Some("measurement"),
        attributes: false,
    }
}

const fn title(key: &'static str, name: &'static str, icon: &'static str) -> Entity {
    Entity {
        key,
        platform: Platform::Sensor,
        name,
        icon,
        unit: None,
        state_class: None,
        attributes: true,
    }
}

pub const ENTITIES: [Entity; 9] = [
    count("games", "Games", "mdi:gamepad-variant", "games"),
    count("copies", "Copies", "mdi:disc", "copies"),
    count("wishlist", "Wishlist", "mdi:heart", "copies"),
    count("backlog", "Backlog", "mdi:bookshelf", "games"),
    Entity {
        attributes: true,
        ..count("playing", "Playing", "mdi:controller", "games")
    },
    title("last_played", "Last played", "mdi:history"),
    title("last_added", "Last added", "mdi:new-box"),
    title("pick", "Current pick", "mdi:dice-5"),
    Entity {
        key: "roll_pick",
        platform: Platform::Button,
        name: "Roll a pick",
        icon: "mdi:dice-5",
        unit: None,
        state_class: None,
        attributes: false,
    },
];

/// The one Memory Card device. `ingress_slug` links its page to the app.
pub fn device(ingress_slug: Option<&str>) -> Value {
    let mut device = json!({
        "name": "Memory Card",
        "model": "Video game collection",
        "sw_version": env!("CARGO_PKG_VERSION"),
    });
    if let Some(slug) = ingress_slug {
        device["configuration_url"] = json!(format!("homeassistant://hassio/ingress/{slug}"));
    }
    device
}

/// The filters a roll asks for: the button's press means the defaults.
pub fn pick_filter(payload: &[u8]) -> Result<SuggestionFilter, serde_json::Error> {
    let trimmed = payload.trim_ascii();
    if trimmed.is_empty() || trimmed == PICK_PRESS.as_bytes() {
        return Ok(SuggestionFilter::default());
    }
    serde_json::from_slice(trimmed)
}

/// Every value, by entity key. Missing titles are `null`, which Home Assistant shows as unknown.
pub fn state(snapshot: &Snapshot) -> Value {
    let last_played = snapshot.last_played.as_ref();
    let last_added = snapshot.last_added.as_ref();
    let pick = snapshot.pick.as_ref();
    json!({
        "games": snapshot.games,
        "copies": snapshot.copies,
        "wishlist": snapshot.wishlist,
        "backlog": snapshot.backlog,
        "playing": snapshot.playing_count,
        "last_played": last_played.map(|game| fit(&game.title)),
        "last_added": last_added.map(|game| fit(&game.title)),
        "pick": pick.map(|game| fit(&game.title)),
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
            "pick": pick.map_or_else(
                || json!({}),
                |game| json!({ "suggested_at": game.suggested_at, "platforms": game.platforms }),
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
    fn every_sensor_has_a_value_and_ids_that_match_its_key() {
        let state = state(&Snapshot {
            games: 0,
            copies: 0,
            wishlist: 0,
            backlog: 0,
            playing: Vec::new(),
            playing_count: 0,
            last_played: None,
            last_added: None,
            pick: None,
        });
        for entity in &ENTITIES {
            if entity.platform == Platform::Sensor {
                assert!(state.get(entity.key).is_some(), "{}", entity.key);
            }
            if entity.attributes {
                assert!(
                    state["attributes"].get(entity.key).is_some(),
                    "{}",
                    entity.key
                );
            }
            assert_eq!(entity.state_class.is_some(), entity.unit.is_some());
        }
        assert_eq!(ENTITIES[0].entity_id(), "sensor.memory_card_games");
        assert_eq!(ENTITIES[8].entity_id(), "button.memory_card_roll_pick");
        assert_eq!(ENTITIES[8].unique_id(), "memory_card_roll_pick");
    }

    #[test]
    fn the_button_rolls_with_defaults_and_automations_send_filters() {
        assert!(pick_filter(b"roll").unwrap().platform.is_empty());
        assert!(pick_filter(b"").unwrap().device.is_empty());
        let filter = pick_filter(br#"{"device": ["switch"], "min_local_players": 2}"#).unwrap();
        assert_eq!(filter.device, ["switch".into()]);
        assert_eq!(filter.min_local_players, Some(2));
        assert!(pick_filter(b"{\"colour\": 1}").is_err());
    }

    #[test]
    fn the_device_links_to_the_app_when_it_knows_the_slug() {
        assert_eq!(
            device(Some("abc123_memory_card"))["configuration_url"],
            "homeassistant://hassio/ingress/abc123_memory_card"
        );
        assert!(device(None).get("configuration_url").is_none());
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
            pick: None,
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
