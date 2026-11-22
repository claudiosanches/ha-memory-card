use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::Deserialize;

/// Address the Supervisor uses to proxy ingress traffic.
pub const SUPERVISOR_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(172, 30, 32, 2));

const DEFAULT_DATA_DIR: &str = "/data";
const DEFAULT_TMP_DIR: &str = "/tmp/memory-card";
const DEFAULT_LISTEN: &str = "0.0.0.0:8099";
const DEFAULT_SUPERVISOR_URL: &str = "http://supervisor";
const DEFAULT_BARCODE_URL: &str = "https://api.upcitemdb.com/prod/trial/lookup";
const DEFAULT_MQTT_PORT: u16 = 1883;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot parse {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("invalid value for {name}: {value:?}")]
    InvalidEnv { name: &'static str, value: String },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warning,
    Error,
}

impl LogLevel {
    pub fn as_directive(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warn",
            Self::Error => "error",
        }
    }
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warning" | "warn" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            _ => Err(()),
        }
    }
}

/// What scanning a copy's NFC tag does to a game whose status counts as finished.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FinishedScan {
    /// Sets it back to playing.
    #[default]
    Playing,
    /// Keeps its status and only logs the session.
    Keep,
}

impl FromStr for FinishedScan {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "playing" => Ok(Self::Playing),
            "keep" => Ok(Self::Keep),
            _ => Err(()),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct IgdbCredentials {
    pub client_id: String,
    pub client_secret: String,
}

impl fmt::Debug for IgdbCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IgdbCredentials")
            .field("client_id", &self.client_id)
            .field("client_secret", &"<redacted>")
            .finish()
    }
}

/// A value kept out of logs and debug output.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MqttBroker {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<Secret>,
}

/// Shape of `/data/options.json` as written by the Supervisor.
#[derive(Debug, Default, Deserialize)]
struct Options {
    igdb_client_id: Option<String>,
    igdb_client_secret: Option<String>,
    log_level: Option<LogLevel>,
    managers: Option<Vec<String>>,
    viewers_track_progress: Option<bool>,
    external_services: Option<bool>,
    sensors: Option<bool>,
    tag_scan_finished: Option<FinishedScan>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub listen: SocketAddr,
    pub trusted_peers: Vec<IpAddr>,
    pub log_level: LogLevel,
    /// `None` without credentials or with external services off.
    pub igdb: Option<IgdbCredentials>,
    /// Home Assistant user IDs allowed to change the collection. While empty,
    /// nobody can.
    pub managers: Vec<String>,
    /// Lets viewers change play status and last played.
    pub viewers_track_progress: bool,
    /// False keeps the app from contacting IGDB or anything else online.
    pub external_services: bool,
    /// Publishes Home Assistant sensors through the MQTT broker.
    pub sensors: bool,
    pub tag_scan_finished: FinishedScan,
    /// Broker named by `MEMORY_CARD_MQTT_*`, used instead of asking the Supervisor.
    pub mqtt: Option<MqttBroker>,
    pub supervisor_url: String,
    /// UPCitemdb's lookup endpoint; tests point it elsewhere.
    pub barcode_url: String,
    /// Home Assistant's API, for tag scans: through the Supervisor inside
    /// Home Assistant, or `MEMORY_CARD_HA_URL` and `MEMORY_CARD_HA_TOKEN`.
    pub home_assistant: Option<HomeAssistantApi>,
    /// Set by the Supervisor inside Home Assistant.
    pub supervisor_token: Option<Secret>,
}

#[derive(Debug, Clone)]
pub struct HomeAssistantApi {
    /// REST base, ending in `/api`.
    pub rest: String,
    pub websocket: String,
    pub token: Secret,
}

impl HomeAssistantApi {
    fn from_env(env: &impl Fn(&str) -> Option<String>, supervisor_url: &str) -> Option<Self> {
        let (base, token) = match (
            non_empty(env("MEMORY_CARD_HA_URL")),
            non_empty(env("MEMORY_CARD_HA_TOKEN")),
        ) {
            (Some(url), Some(token)) => (url.trim_end_matches('/').to_owned(), token),
            _ => (
                format!("{supervisor_url}/core"),
                non_empty(env("SUPERVISOR_TOKEN"))?,
            ),
        };
        let websocket = base
            .replacen("https://", "wss://", 1)
            .replacen("http://", "ws://", 1);
        // The Supervisor proxy serves the websocket at /core/websocket, Home Assistant at /api/websocket.
        let websocket = if base.ends_with("/core") {
            format!("{websocket}/websocket")
        } else {
            format!("{websocket}/api/websocket")
        };
        Some(Self {
            rest: format!("{base}/api"),
            websocket,
            token: Secret(token),
        })
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::load(|name| std::env::var(name).ok())
    }

    /// Reads options.json from the data dir, then applies `MEMORY_CARD_*` overrides.
    pub fn load(env: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let data_dir =
            PathBuf::from(env("MEMORY_CARD_DATA_DIR").unwrap_or(DEFAULT_DATA_DIR.into()));
        let tmp_dir = PathBuf::from(env("MEMORY_CARD_TMP_DIR").unwrap_or(DEFAULT_TMP_DIR.into()));
        let options = read_options(&data_dir.join("options.json"))?;

        let listen_raw = env("MEMORY_CARD_LISTEN").unwrap_or(DEFAULT_LISTEN.into());
        let listen = listen_raw.parse().map_err(|_| ConfigError::InvalidEnv {
            name: "MEMORY_CARD_LISTEN",
            value: listen_raw.clone(),
        })?;

        let trusted_peers = match env("MEMORY_CARD_TRUSTED_PEER") {
            None => vec![SUPERVISOR_IP],
            Some(raw) => raw
                .split(',')
                .map(|part| {
                    part.trim().parse().map_err(|_| ConfigError::InvalidEnv {
                        name: "MEMORY_CARD_TRUSTED_PEER",
                        value: raw.clone(),
                    })
                })
                .collect::<Result<_, _>>()?,
        };

        let log_level = match env("MEMORY_CARD_LOG_LEVEL") {
            None => options.log_level.unwrap_or_default(),
            Some(raw) => raw.parse().map_err(|()| ConfigError::InvalidEnv {
                name: "MEMORY_CARD_LOG_LEVEL",
                value: raw.clone(),
            })?,
        };

        let external_services = flag(
            &env,
            "MEMORY_CARD_EXTERNAL_SERVICES",
            options.external_services,
            true,
        )?;
        let client_id = non_empty(env("MEMORY_CARD_IGDB_CLIENT_ID").or(options.igdb_client_id));
        let client_secret =
            non_empty(env("MEMORY_CARD_IGDB_CLIENT_SECRET").or(options.igdb_client_secret));
        let igdb = match (client_id, client_secret) {
            (Some(client_id), Some(client_secret)) if external_services => Some(IgdbCredentials {
                client_id,
                client_secret,
            }),
            _ => None,
        };

        let managers = match env("MEMORY_CARD_MANAGERS") {
            Some(raw) => raw.split(',').map(str::to_owned).collect(),
            None => options.managers.unwrap_or_default(),
        };
        let managers = managers
            .into_iter()
            .filter_map(|id| non_empty(Some(id)))
            .collect();

        let viewers_track_progress = flag(
            &env,
            "MEMORY_CARD_VIEWERS_TRACK_PROGRESS",
            options.viewers_track_progress,
            false,
        )?;

        let sensors = flag(&env, "MEMORY_CARD_SENSORS", options.sensors, true)?;
        let mqtt = match non_empty(env("MEMORY_CARD_MQTT_HOST")) {
            None => None,
            Some(host) => {
                let port = match env("MEMORY_CARD_MQTT_PORT") {
                    None => DEFAULT_MQTT_PORT,
                    Some(raw) => raw.trim().parse().map_err(|_| ConfigError::InvalidEnv {
                        name: "MEMORY_CARD_MQTT_PORT",
                        value: raw.clone(),
                    })?,
                };
                Some(MqttBroker {
                    host,
                    port,
                    username: non_empty(env("MEMORY_CARD_MQTT_USERNAME")),
                    password: non_empty(env("MEMORY_CARD_MQTT_PASSWORD")).map(Secret),
                })
            }
        };
        let supervisor_url = env("MEMORY_CARD_SUPERVISOR_URL")
            .unwrap_or(DEFAULT_SUPERVISOR_URL.into())
            .trim_end_matches('/')
            .to_owned();

        let home_assistant = HomeAssistantApi::from_env(&env, &supervisor_url);
        let tag_scan_finished = match env("MEMORY_CARD_TAG_SCAN_FINISHED") {
            None => options.tag_scan_finished.unwrap_or_default(),
            Some(raw) => raw.parse().map_err(|()| ConfigError::InvalidEnv {
                name: "MEMORY_CARD_TAG_SCAN_FINISHED",
                value: raw.clone(),
            })?,
        };

        Ok(Self {
            home_assistant,
            data_dir,
            tmp_dir,
            listen,
            trusted_peers,
            log_level,
            igdb,
            managers,
            viewers_track_progress,
            external_services,
            sensors,
            tag_scan_finished,
            mqtt,
            supervisor_url,
            barcode_url: env("MEMORY_CARD_BARCODE_URL").unwrap_or(DEFAULT_BARCODE_URL.into()),
            supervisor_token: non_empty(env("SUPERVISOR_TOKEN")).map(Secret),
        })
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("db").join("memory-card.sqlite")
    }

    pub fn pre_migration_dir(&self) -> PathBuf {
        self.data_dir.join("db").join("pre-migration")
    }

    pub fn covers_dir(&self) -> PathBuf {
        self.data_dir.join("covers")
    }
}

fn read_options(path: &Path) -> Result<Options, ConfigError> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|source| ConfigError::Parse {
            path: path.to_owned(),
            source,
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Options::default()),
        Err(source) => Err(ConfigError::Read {
            path: path.to_owned(),
            source,
        }),
    }
}

/// A boolean option, overridden by `name` (`true`, `false`, `1` or `0`).
fn flag(
    env: impl Fn(&str) -> Option<String>,
    name: &'static str,
    option: Option<bool>,
    default: bool,
) -> Result<bool, ConfigError> {
    let Some(raw) = env(name) else {
        return Ok(option.unwrap_or(default));
    };
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(ConfigError::InvalidEnv { name, value: raw }),
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn load_with(dir: &Path, vars: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let mut map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        map.insert("MEMORY_CARD_DATA_DIR".into(), dir.display().to_string());
        Config::load(|name| map.get(name).cloned())
    }

    #[test]
    fn home_assistant_api_comes_from_the_supervisor_or_the_environment() {
        let dir = tempfile::tempdir().unwrap();
        let config = load_with(dir.path(), &[("SUPERVISOR_TOKEN", "sup")]).unwrap();
        let api = config.home_assistant.unwrap();
        assert_eq!(api.rest, "http://supervisor/core/api");
        assert_eq!(api.websocket, "ws://supervisor/core/websocket");

        let config = load_with(
            dir.path(),
            &[
                ("MEMORY_CARD_HA_URL", "http://ha.local:8123/"),
                ("MEMORY_CARD_HA_TOKEN", "t"),
            ],
        )
        .unwrap();
        let api = config.home_assistant.unwrap();
        assert_eq!(api.rest, "http://ha.local:8123/api");
        assert_eq!(api.websocket, "ws://ha.local:8123/api/websocket");

        assert!(load_with(dir.path(), &[]).unwrap().home_assistant.is_none());
    }

    #[test]
    fn defaults_without_options_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = load_with(dir.path(), &[]).unwrap();
        assert_eq!(config.trusted_peers, vec![SUPERVISOR_IP]);
        assert_eq!(config.listen, "0.0.0.0:8099".parse().unwrap());
        assert_eq!(config.log_level, LogLevel::Info);
        assert!(config.igdb.is_none());
        assert!(config.managers.is_empty());
        assert!(!config.viewers_track_progress);
        assert!(config.external_services);
        assert_eq!(config.tag_scan_finished, FinishedScan::Playing);
        assert_eq!(config.db_path(), dir.path().join("db/memory-card.sqlite"));
    }

    #[test]
    fn reads_supervisor_options() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("options.json"),
            r#"{"igdb_client_id":"id","igdb_client_secret":"secret","log_level":"warning","managers":[" abc123 ",""],"viewers_track_progress":true,"tag_scan_finished":"keep"}"#,
        )
        .unwrap();
        let config = load_with(dir.path(), &[]).unwrap();
        assert_eq!(config.log_level, LogLevel::Warning);
        assert_eq!(config.log_level.as_directive(), "warn");
        assert_eq!(config.managers, vec!["abc123"]);
        assert!(config.viewers_track_progress);
        assert_eq!(config.tag_scan_finished, FinishedScan::Keep);
        let igdb = config.igdb.unwrap();
        assert_eq!(igdb.client_id, "id");
        assert!(!format!("{igdb:?}").contains("secret\""));
    }

    #[test]
    fn env_overrides_options() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("options.json"),
            r#"{"igdb_client_id":"id","igdb_client_secret":"","log_level":"error","managers":["abc"]}"#,
        )
        .unwrap();
        let config = load_with(
            dir.path(),
            &[
                ("MEMORY_CARD_LOG_LEVEL", "debug"),
                ("MEMORY_CARD_TRUSTED_PEER", "127.0.0.1, ::1"),
                ("MEMORY_CARD_LISTEN", "127.0.0.1:18099"),
                ("MEMORY_CARD_MANAGERS", "one, two,"),
                ("MEMORY_CARD_VIEWERS_TRACK_PROGRESS", "1"),
            ],
        )
        .unwrap();
        assert_eq!(config.log_level, LogLevel::Debug);
        assert_eq!(config.trusted_peers.len(), 2);
        assert_eq!(config.managers, vec!["one", "two"]);
        assert!(config.viewers_track_progress);
        assert!(config.igdb.is_none(), "empty secret disables IGDB");
    }

    #[test]
    fn external_services_off_ignores_igdb_credentials() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("options.json"),
            r#"{"igdb_client_id":"id","igdb_client_secret":"secret","external_services":false}"#,
        )
        .unwrap();
        let config = load_with(dir.path(), &[]).unwrap();
        assert!(!config.external_services);
        assert!(config.igdb.is_none());

        let config = load_with(dir.path(), &[("MEMORY_CARD_EXTERNAL_SERVICES", "true")]).unwrap();
        assert!(config.igdb.is_some());
    }

    #[test]
    fn sensors_use_the_broker_from_env_or_ask_the_supervisor() {
        let dir = tempfile::tempdir().unwrap();
        let config = load_with(dir.path(), &[]).unwrap();
        assert!(config.sensors);
        assert!(config.mqtt.is_none());
        assert!(config.supervisor_token.is_none());
        assert_eq!(config.supervisor_url, "http://supervisor");

        std::fs::write(dir.path().join("options.json"), r#"{"sensors":false}"#).unwrap();
        let config = load_with(
            dir.path(),
            &[
                ("SUPERVISOR_TOKEN", "token-1"),
                ("MEMORY_CARD_MQTT_HOST", "localhost"),
                ("MEMORY_CARD_MQTT_PASSWORD", "hunter2"),
            ],
        )
        .unwrap();
        assert!(!config.sensors);
        assert_eq!(config.supervisor_token.unwrap().expose(), "token-1");
        let mqtt = config.mqtt.unwrap();
        assert_eq!((mqtt.host.as_str(), mqtt.port), ("localhost", 1883));
        assert!(!format!("{mqtt:?}").contains("hunter2"));

        let err = load_with(
            dir.path(),
            &[
                ("MEMORY_CARD_MQTT_HOST", "localhost"),
                ("MEMORY_CARD_MQTT_PORT", "x"),
            ],
        )
        .unwrap_err();
        assert!(matches!(err, ConfigError::InvalidEnv { .. }));
    }

    #[test]
    fn rejects_bad_env() {
        let dir = tempfile::tempdir().unwrap();
        let err = load_with(dir.path(), &[("MEMORY_CARD_TRUSTED_PEER", "nope")]).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidEnv { .. }));
        let err = load_with(dir.path(), &[("MEMORY_CARD_EXTERNAL_SERVICES", "maybe")]).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidEnv { .. }));
        let err =
            load_with(dir.path(), &[("MEMORY_CARD_TAG_SCAN_FINISHED", "paused")]).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidEnv { .. }));
    }

    #[test]
    fn rejects_malformed_options() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("options.json"), "{").unwrap();
        let err = load_with(dir.path(), &[]).unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }
}
