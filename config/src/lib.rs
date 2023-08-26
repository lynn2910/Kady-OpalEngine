use serde::{ Deserialize, Serialize };
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
    pub pid: String,
    pub api: ApiConfig,
    pub security: SecurityConfig,
    pub status: StatusConfig
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
    pub close_timeout: u64
}

/// Contain security settings
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SecurityConfig {
    pub archive_path: String
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


pub fn load_from(path: String) -> Result<Config> {
    let content: String = match std::fs::read_to_string(path.clone()) {
        Ok(content) => content,
        Err(_) => return Err(error::Error::Config(error::ConfigError::CannotReadFile(path)))
    };

    match toml::from_str(content.as_str()) {
        Ok(config) => Ok(config),
        Err(e) => {
            eprintln!("{}", e);
            Err(error::Error::Config(error::ConfigError::InvalidFile(format!("{path:?} ({e:?})"))))
        }
    }
}