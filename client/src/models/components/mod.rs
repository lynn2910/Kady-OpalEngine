pub mod embed;
pub mod sticker;
pub mod message_components;

use serde::{ Serialize, Deserialize };
use serde_json::{json, Value};
use crate::models::Snowflake;
use crate::models::user::User;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Color(pub u64);

impl Color {
    pub const EMBED_COLOR: Self = Self(2829617);

    pub fn from_hex(hex: impl Into<String>) -> Self {
        Self(u64::from_str_radix(&hex.into().replace('#', ""), 16).unwrap_or(0))
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self((r as u64) << 16 | (g as u64) << 8 | b as u64)
    }
}

impl From<String> for Color {
    fn from(color: String) -> Self {
        Self::from_hex(color.replace('#', ""))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Emoji {
    pub id: Option<Snowflake>,
    pub name: String,
    /// Roles allowed to use this emoji
    pub roles: Vec<Snowflake>,
    /// The user that created this emoji
    pub user: Option<User>,
    /// Whether this emoji must be wrapped in colons
    pub require_colons: Option<bool>,
    /// Whether this emoji is managed
    pub managed: Option<bool>,
    /// Whether this emoji is animated
    pub animated: Option<bool>,
    /// Whether this emoji is available
    pub available: Option<bool>,
}

impl From<&str> for Emoji {
    fn from(value: &str) -> Self {
        Self::new(None, value.to_string())
    }
}

impl From<String> for Emoji {
    fn from(value: String) -> Self {
        Self::new(None, value)
    }
}

impl Emoji {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name
        })
    }

    pub fn new(id: Option<Snowflake>, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            roles: vec![],
            user: None,
            require_colons: None,
            managed: None,
            animated: None,
            available: None,
        }
    }
}
pub(crate) mod timestamp_serde {
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
    use chrono::{DateTime, TimeZone, Utc};

    pub fn serialize<S>(
        timestamp: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match timestamp {
            Some(time) => serializer.serialize_str(&time.to_rfc3339()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let str_val: Option<String> = Deserialize::deserialize(deserializer)?;

        str_val
            .map(|s| DateTime::parse_from_rfc3339(&s)
                .map_err(serde::de::Error::custom)
                .map(|dt| dt.with_timezone(&Utc)))
            .transpose()
    }
}
