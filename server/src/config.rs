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

/// Shape of `/data/options.json` as written by the Supervisor.
#[derive(Debug, Default, Deserialize)]
struct Options {
    igdb_client_id: Option<String>,
    igdb_client_secret: Option<String>,
    log_level: Option<LogLevel>,
    managers: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub listen: SocketAddr,
    pub trusted_peers: Vec<IpAddr>,
    pub log_level: LogLevel,
    pub igdb: Option<IgdbCredentials>,
    /// Home Assistant user IDs allowed to change the collection. Empty means
    /// every ingress user may, and identity is not required.
    pub managers: Vec<String>,
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

        let client_id = non_empty(env("MEMORY_CARD_IGDB_CLIENT_ID").or(options.igdb_client_id));
        let client_secret =
            non_empty(env("MEMORY_CARD_IGDB_CLIENT_SECRET").or(options.igdb_client_secret));
        let igdb = match (client_id, client_secret) {
            (Some(client_id), Some(client_secret)) => Some(IgdbCredentials {
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

        Ok(Self {
            data_dir,
            tmp_dir,
            listen,
            trusted_peers,
            log_level,
            igdb,
            managers,
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
    fn defaults_without_options_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = load_with(dir.path(), &[]).unwrap();
        assert_eq!(config.trusted_peers, vec![SUPERVISOR_IP]);
        assert_eq!(config.listen, "0.0.0.0:8099".parse().unwrap());
        assert_eq!(config.log_level, LogLevel::Info);
        assert!(config.igdb.is_none());
        assert!(config.managers.is_empty());
        assert_eq!(config.db_path(), dir.path().join("db/memory-card.sqlite"));
    }

    #[test]
    fn reads_supervisor_options() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("options.json"),
            r#"{"igdb_client_id":"id","igdb_client_secret":"secret","log_level":"warning","managers":[" abc123 ",""]}"#,
        )
        .unwrap();
        let config = load_with(dir.path(), &[]).unwrap();
        assert_eq!(config.log_level, LogLevel::Warning);
        assert_eq!(config.log_level.as_directive(), "warn");
        assert_eq!(config.managers, vec!["abc123"]);
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
            ],
        )
        .unwrap();
        assert_eq!(config.log_level, LogLevel::Debug);
        assert_eq!(config.trusted_peers.len(), 2);
        assert_eq!(config.managers, vec!["one", "two"]);
        assert!(config.igdb.is_none(), "empty secret disables IGDB");
    }

    #[test]
    fn rejects_bad_env() {
        let dir = tempfile::tempdir().unwrap();
        let err = load_with(dir.path(), &[("MEMORY_CARD_TRUSTED_PEER", "nope")]).unwrap_err();
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
