pub mod events;
pub mod user;
pub mod guild;
pub mod channel;
pub mod message;
pub mod interaction;
pub mod components;
pub mod presence;

use std::fmt::Display;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use error::{Error, ModelError, Result};
use crate::manager::http::HttpRessource;

/// Represent a Discord snowflake
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, sqlx::FromRow, sqlx::Decode)]
pub struct Snowflake(pub String);

impl sqlx::Type<sqlx::MySql> for Snowflake {
    fn type_info() -> sqlx::mysql::MySqlTypeInfo {
        <String as sqlx::Type<sqlx::MySql>>::type_info()
    }
}

pub const DISCORD_EPOCH: u64 = 1420070400000;

impl Snowflake {
    pub fn get_timestamp(&self) -> Result<chrono::DateTime<Utc>> {
        let timestamp = match self.0.parse::<u64>() {
            Ok(timestamp) => timestamp,
            Err(_) => return Err(Error::Model(ModelError::InvalidSnowflake("Failed to parse snowflake".into())))
        };

        let timestamp = (timestamp >> 22) + DISCORD_EPOCH;

        // Convertir le timestamp en DateTime<Utc>
        let datetime = match Utc.timestamp_millis_opt(timestamp as i64).single() {
            Some(datetime) => datetime,
            None => return Err(Error::Model(ModelError::InvalidSnowflake("Failed to convert timestamp to DateTime<Utc>".into())))
        };

        Ok(datetime)
    }
}

impl From<&str> for Snowflake {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}
impl From<String> for Snowflake {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&String> for Snowflake {
    fn from(s: &String) -> Self {
        Self(s.into())
    }
}

impl From<&Snowflake> for Snowflake {
    fn from(s: &Snowflake) -> Self {
        s.clone()
    }
}

impl From<Snowflake> for String {
    fn from(value: Snowflake) -> Self {
        value.0
    }
}

impl Display for Snowflake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.to_string().fmt(f)
    }
}

impl HttpRessource for Snowflake {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        match raw.as_str() {
            Some(snowflake) => Ok(Self(snowflake.into())),
            None => Err(Error::Model(ModelError::InvalidSnowflake(format!("Failed to parse snowflake: {raw:?}"))))
        }
    }
}

