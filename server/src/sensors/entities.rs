//! The entities Home Assistant gets and the state they read, whether MQTT or
//! the integration carries them.

use serde::Serialize;
use serde_json::{Value, json};

use std::collections::HashSet;

use crate::db::sensors::{PersonPlay, Snapshot};
use crate::db::suggestions::SuggestionFilter;

/// Home Assistant drops longer states.
const MAX_STATE: usize = 255;
/// The part of a person's user ID their sensors' keys use.
const PERSON_KEY_MAX: usize = 32;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entity {
    /// Names its value in the state, and its unique ID.
    pub key: String,
    pub platform: Platform,
    pub name: String,
    pub icon: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<&'static str>,
    /// Counts keep long-term statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_class: Option<&'static str>,
    /// The state also carries `attributes.<key>`.
    pub attributes: bool,
    /// What follows `memory_card_` in the entity ID Home Assistant suggests.
    pub object_id: String,
    /// A person's own sensor: their name, and which of theirs it is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person: Option<PersonSensor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersonSensor {
    pub name: String,
    /// `backlog`, `playing` or `last_played`.
    pub kind: &'static str,
}

impl Entity {
    pub fn unique_id(&self) -> String {
        format!("memory_card_{}", self.key)
    }

    pub fn entity_id(&self) -> String {
        format!("{}.memory_card_{}", self.platform.as_str(), self.object_id)
    }
}

/// What kind of value an entity shows.
#[derive(Clone, Copy)]
enum Shape {
    /// A number of games or copies, with a unit.
    Count(&'static str),
    /// A count with a list of titles in its attributes.
    CountWithTitles(&'static str),
    /// A title with details in its attributes.
    Title,
    Button,
}

struct Spec {
    key: &'static str,
    name: &'static str,
    icon: &'static str,
    shape: Shape,
}

const HOUSEHOLD: [Spec; 7] = [
    Spec {
        key: "games",
        name: "Games",
        icon: "mdi:gamepad-variant",
        shape: Shape::Count("games"),
    },
    Spec {
        key: "copies",
        name: "Copies",
        icon: "mdi:disc",
        shape: Shape::Count("copies"),
    },
    Spec {
        key: "wishlist",
        name: "Wishlist",
        icon: "mdi:heart",
        shape: Shape::Count("copies"),
    },
    Spec {
        key: "playing",
        name: "Playing",
        icon: "mdi:controller",
        shape: Shape::CountWithTitles("games"),
    },
    Spec {
        key: "last_added",
        name: "Last added",
        icon: "mdi:new-box",
        shape: Shape::Title,
    },
    Spec {
        key: "pick",
        name: "Current pick",
        icon: "mdi:dice-5",
        shape: Shape::Title,
    },
    Spec {
        key: "roll_pick",
        name: "Roll a pick",
        icon: "mdi:dice-5",
        shape: Shape::Button,
    },
];

const PERSONAL: [Spec; 3] = [
    Spec {
        key: "backlog",
        name: "backlog",
        icon: "mdi:bookshelf",
        shape: Shape::Count("games"),
    },
    Spec {
        key: "playing",
        name: "playing",
        icon: "mdi:controller",
        shape: Shape::CountWithTitles("games"),
    },
    Spec {
        key: "last_played",
        name: "last played",
        icon: "mdi:history",
        shape: Shape::Title,
    },
];

fn entity(spec: &Spec, key: String, name: String, object_id: String) -> Entity {
    let (platform, unit, attributes) = match spec.shape {
        Shape::Count(unit) => (Platform::Sensor, Some(unit), false),
        Shape::CountWithTitles(unit) => (Platform::Sensor, Some(unit), true),
        Shape::Title => (Platform::Sensor, None, true),
        Shape::Button => (Platform::Button, None, false),
    };
    Entity {
        key,
        platform,
        name,
        icon: spec.icon,
        unit,
        state_class: unit.map(|_| "measurement"),
        attributes,
        object_id,
        person: None,
    }
}

/// A person's sensors and the keys that tie them to the state.
struct Person<'a> {
    play: &'a PersonPlay,
    /// From their user ID, so the sensors keep their IDs through a rename.
    key: String,
    /// From their name, for the entity IDs Home Assistant suggests.
    object_id: String,
}

/// Each person with a usable key, their sensors named after them.
fn people(snapshot: &Snapshot) -> Vec<Person<'_>> {
    let (mut keys, mut objects) = (HashSet::new(), HashSet::new());
    let mut people = Vec::new();
    for play in &snapshot.people {
        let key: String = play
            .id
            .to_ascii_lowercase()
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .take(PERSON_KEY_MAX)
            .collect();
        if key.is_empty() || !keys.insert(key.clone()) {
            continue;
        }
        let base = play
            .name
            .as_deref()
            .map(slug)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| key.clone());
        let mut object_id = base.clone();
        let mut n = 2;
        while !objects.insert(object_id.clone()) {
            object_id = format!("{base}_{n}");
            n += 1;
        }
        people.push(Person {
            play,
            key: format!("person_{key}"),
            object_id,
        });
    }
    people
}

/// Lowercase ASCII letters and digits joined by single underscores.
fn slug(name: &str) -> String {
    name.to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// The household's entities, then each person's.
pub fn entities(snapshot: &Snapshot) -> Vec<Entity> {
    let mut entities: Vec<Entity> = HOUSEHOLD
        .iter()
        .map(|spec| entity(spec, spec.key.into(), spec.name.into(), spec.key.into()))
        .collect();
    for person in people(snapshot) {
        let name = person.play.name.clone().unwrap_or_else(|| "Someone".into());
        for spec in &PERSONAL {
            entities.push(Entity {
                person: Some(PersonSensor {
                    name: name.clone(),
                    kind: spec.key,
                }),
                ..entity(
                    spec,
                    format!("{}_{}", person.key, spec.key),
                    format!("{name} {}", spec.name),
                    format!("{}_{}", person.object_id, spec.key),
                )
            });
        }
    }
    entities
}

/// The device the entities belong to.
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

/// A roll from Home Assistant: whose backlog it picks from, and the filters.
#[derive(Debug, Default)]
pub struct RollRequest {
    /// A Home Assistant user ID.
    pub person: Option<String>,
    pub filter: SuggestionFilter,
}

/// Reads a roll: nothing means the defaults, otherwise a JSON object with an
/// optional `person` beside the filters.
pub fn roll_request(payload: &[u8]) -> Result<RollRequest, serde_json::Error> {
    let trimmed = payload.trim_ascii();
    if trimmed.is_empty() {
        return Ok(RollRequest::default());
    }
    let mut fields: serde_json::Map<String, Value> = serde_json::from_slice(trimmed)?;
    let person = match fields.remove("person") {
        None | Some(Value::Null) => None,
        Some(Value::String(person)) => Some(person),
        Some(_) => return Err(serde::de::Error::custom("person is a user ID")),
    };
    Ok(RollRequest {
        person,
        filter: serde_json::from_value(Value::Object(fields))?,
    })
}

/// Every value, by entity key. Missing titles are `null`, which Home Assistant shows as unknown.
pub fn state(snapshot: &Snapshot) -> Value {
    let last_added = snapshot.last_added.as_ref();
    let pick = snapshot.pick.as_ref();
    let mut state = json!({
        "games": snapshot.games,
        "copies": snapshot.copies,
        "wishlist": snapshot.wishlist,
        "playing": snapshot.playing_count,
        "last_added": last_added.map(|game| fit(&game.title)),
        "pick": pick.map(|game| fit(&game.title)),
        "attributes": {
            "playing": { "games": snapshot.playing },
            "last_added": last_added.map_or_else(
                || json!({}),
                |game| json!({ "added_at": game.added_at, "platform": game.platform }),
            ),
            "pick": pick.map_or_else(
                || json!({}),
                |game| json!({
                    "suggested_at": game.suggested_at,
                    "platforms": game.platforms,
                    "person": game.person,
                }),
            ),
        },
    });
    for person in people(snapshot) {
        let play = person.play;
        let last_played = play.last_played.as_ref();
        let key = |kind: &str| format!("{}_{kind}", person.key);
        state[key("backlog")] = json!(play.backlog);
        state[key("playing")] = json!(play.playing_count);
        state[key("last_played")] = json!(last_played.map(|game| fit(&game.title)));
        state["attributes"][key("playing")] = json!({ "games": play.playing });
        state["attributes"][key("last_played")] = last_played.map_or_else(
            || json!({}),
            |game| json!({ "date": game.date, "platform": game.platform }),
        );
    }
    state
}

fn fit(title: &str) -> String {
    title.chars().take(MAX_STATE).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::sensors::{LastAdded, LastPlayed};

    fn person(id: &str, name: Option<&str>) -> PersonPlay {
        PersonPlay {
            id: id.into(),
            name: name.map(Into::into),
            backlog: 2,
            playing: vec!["Celeste".into()],
            playing_count: 1,
            last_played: Some(LastPlayed {
                title: "x".repeat(300),
                date: "2026-11-20".into(),
                platform: None,
            }),
        }
    }

    #[test]
    fn every_sensor_has_a_value_and_ids_that_match_its_key() {
        let snapshot = Snapshot {
            people: vec![person("0123ABCDef", Some("Ana Lúcia"))],
            ..Snapshot::default()
        };
        let state = state(&snapshot);
        let entities = entities(&snapshot);
        assert_eq!(entities.len(), HOUSEHOLD.len() + PERSONAL.len());
        for entity in &entities {
            if entity.platform == Platform::Sensor {
                assert!(state.get(&entity.key).is_some(), "{}", entity.key);
            }
            if entity.attributes {
                assert!(
                    state["attributes"].get(&entity.key).is_some(),
                    "{}",
                    entity.key
                );
            }
            assert_eq!(entity.state_class.is_some(), entity.unit.is_some());
        }
        assert_eq!(entities[0].entity_id(), "sensor.memory_card_games");
        assert_eq!(entities[6].entity_id(), "button.memory_card_roll_pick");
        assert_eq!(entities[6].unique_id(), "memory_card_roll_pick");

        let backlog = &entities[7];
        assert_eq!(backlog.key, "person_0123abcdef_backlog");
        assert_eq!(backlog.unique_id(), "memory_card_person_0123abcdef_backlog");
        assert_eq!(backlog.entity_id(), "sensor.memory_card_ana_l_cia_backlog");
        assert_eq!(backlog.name, "Ana Lúcia backlog");
        assert_eq!(
            backlog.person,
            Some(PersonSensor {
                name: "Ana Lúcia".into(),
                kind: "backlog",
            })
        );
    }

    #[test]
    fn people_get_unique_keys_and_entity_ids() {
        let snapshot = Snapshot {
            people: vec![
                person("aa11", Some("Sam")),
                person("bb22", Some("Sam")),
                person("cc33", None),
                person("---", Some("Nobody")),
                person("AA11", Some("Twin")),
            ],
            ..Snapshot::default()
        };
        let ids: Vec<String> = entities(&snapshot)
            .iter()
            .filter(|e| e.key.ends_with("_backlog"))
            .map(Entity::entity_id)
            .collect();
        assert_eq!(
            ids,
            [
                "sensor.memory_card_sam_backlog",
                "sensor.memory_card_sam_2_backlog",
                "sensor.memory_card_cc33_backlog",
            ]
        );
    }

    #[test]
    fn a_roll_names_a_person_beside_its_filters() {
        let empty = roll_request(b"").unwrap();
        assert!(empty.person.is_none() && empty.filter.device.is_empty());
        let request =
            roll_request(br#"{"person": "abc", "device": ["switch"], "min_local_players": 2}"#)
                .unwrap();
        assert_eq!(request.person.as_deref(), Some("abc"));
        assert_eq!(request.filter.device, ["switch".into()]);
        assert_eq!(request.filter.min_local_players, Some(2));
        assert!(roll_request(b"roll").is_err());
        assert!(roll_request(br#"{"person": 1}"#).is_err());
        assert!(roll_request(b"{\"colour\": 1}").is_err());
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
            playing: vec!["Celeste".into()],
            playing_count: 1,
            people: vec![person("ana", Some("Ana"))],
            ..Snapshot::default()
        };
        let state = state(&snapshot);
        assert_eq!(state["games"], 3);
        assert_eq!(state["playing"], 1);
        assert_eq!(state["attributes"]["playing"]["games"], json!(["Celeste"]));
        assert_eq!(state["person_ana_backlog"], 2);
        assert_eq!(state["person_ana_playing"], 1);
        assert_eq!(
            state["person_ana_last_played"].as_str().unwrap().len(),
            MAX_STATE
        );
        assert_eq!(
            state["attributes"]["person_ana_last_played"]["date"],
            "2026-11-20"
        );
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
