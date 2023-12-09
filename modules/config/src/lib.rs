use std::path::PathBuf;
use serde::{Deserialize, Serialize };
use client::models::presence::Activity;
use client::typemap::Type;

use error::Result;

/// Contain the full configuration of the client
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub intents: u64,
    pub version: String,
    pub build: String,
    pub langs: String,
    pub dynamic_requests: String,
    pub memory_report_path: String,
    pub logs_path: String,
    pub core_path: String,
    pub pid: String,
    pub api: ApiConfig,
    pub security: SecurityConfig,
    pub status: StatusConfig,
    pub client: ClientConfig,
    pub core: CoreConfig,
    pub database: DatabaseConfig
}

impl Config {
    pub fn save(path: String, config: &Config) -> Result<()> {
        let content: String = match toml::to_string(&config) {
            Ok(content) => content,
            Err(_) => return Err(error::Error::Config(error::ConfigError::CannotWriteFile(path)))
        };

        match std::fs::write(path.clone(), content) {
            Ok(_) => Ok(()),
            Err(_) => Err(error::Error::Config(error::ConfigError::CannotWriteFile(path)))
        }
    }

    pub fn reload(&mut self, path: String) -> Result<()> {
        let config = load_from(path)?;

        self.intents = config.intents;
        self.api = config.api;

        Ok(())
    }
}

impl Type for Config {
    type Value = Self;
}

/// Contain api settings
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApiConfig {
    pub retry_limit: u64,
    pub close_timeout: u64,
    pub declared_files: Vec<(String, String, String)>
}

/// Contain security settings
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SecurityConfig {
    pub archive_path: String,
    pub discord_token: String
}

/// Contain status settings
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StatusConfig {
    pub alternate: Vec<Activity>,
    pub interval: u64,
    pub dev: Activity,
    pub maintenance: Activity,
    pub unavailable: Activity,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClientConfig {
    pub guild_add_channel: Option<String>,
    pub guild_remove_channel: Option<String>,
    pub most_used_commands: Vec<String>,
    pub invite_required_permissions: u64,
    pub invite_scope: String,
    pub support_url: String,
    pub website: String,
    pub top_gg: String,
    pub suggestion_channel: String,
    pub issue_channel: String,
    pub review_channel: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CoreConfig {
    pub cargo_version: String,
    pub rustc_version: String,
    pub rustup_version: String
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String
}

pub fn load_from(path: impl Into<PathBuf>) -> Result<Config> {
    let path = path.into();

    let content: String = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => return Err(error::Error::Config(error::ConfigError::CannotReadFile(path.to_string_lossy().to_string())))
    };

    match toml::from_str(content.as_str()) {
        Ok(config) => Ok(config),
        Err(e) => {
            eprintln!("{}", e);
            Err(error::Error::Config(error::ConfigError::InvalidFile(format!("{path:?} ({e:?})"))))
        }
    }
}