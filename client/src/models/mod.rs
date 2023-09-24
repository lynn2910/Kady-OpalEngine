pub mod events;
pub mod user;
pub mod guild;
pub mod channel;
pub mod message;
pub mod interaction;
pub mod components;
pub mod presence;

use std::fmt::Display;
use std::num::ParseIntError;
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use error::{Error, ModelError, Result};
use crate::manager::http::HttpRessource;
use crate::models::guild::GuildId;
use crate::models::user::UserId;

/// Represent a Discord snowflake
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, sqlx::FromRow, sqlx::Decode)]
pub struct Snowflake(pub String);

/// Contain every informations given by a Snowflake from Discord
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SnowflakeInfo {
    pub timestamp: DateTime<Utc>,
    pub data_center_id: u16,
    pub worker_id: u16,
    pub sequence: u16,
}

impl TryFrom<Snowflake> for SnowflakeInfo {
    type Error = ParseIntError;

    fn try_from(snowflake: Snowflake) -> std::result::Result<Self, Self::Error> {
        let snowflake_num = snowflake.to_string().parse::<u64>()?;

        // Constantes pour extraire les composants du snowflake
        const DATA_CENTER_MASK: u64 = 0x00000000003E0000;
        const WORKER_MASK: u64 = 0x000000000001F000;
        const SEQUENCE_MASK: u64 = 0x0000000000000FFF;

        let milliseconds = (snowflake_num >> 22) as i64 + 1420070400000;
        let timestamp = Utc.timestamp_opt(milliseconds / 1000, 0)
            .single()
            .unwrap_or(Utc::now() - Duration::seconds(60));

        let data_center_id = (snowflake_num >> 17) & DATA_CENTER_MASK;
        let worker_id = (snowflake_num >> 12) & WORKER_MASK;
        let sequence = snowflake_num & SEQUENCE_MASK;

        Ok(SnowflakeInfo {
            timestamp,
            data_center_id: (data_center_id >> 17) as u16,
            worker_id: (worker_id >> 12) as u16,
            sequence: sequence as u16,
        })
    }
}

impl TryFrom<GuildId> for SnowflakeInfo {
    type Error = ParseIntError;

    fn try_from(value: GuildId) -> std::result::Result<Self, Self::Error> {
        value.0.try_into()
    }
}


impl TryFrom<UserId> for SnowflakeInfo {
    type Error = ParseIntError;

    fn try_from(value: UserId) -> std::result::Result<Self, Self::Error> {
        value.0.try_into()
    }
}


impl sqlx::Type<sqlx::MySql> for Snowflake {
    fn type_info() -> sqlx::mysql::MySqlTypeInfo {
        <String as sqlx::Type<sqlx::MySql>>::type_info()
    }
}

pub const DISCORD_EPOCH: u64 = 1420070400000;

impl Snowflake {
    pub fn get_timestamp(&self) -> Result<DateTime<Utc>> {
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

