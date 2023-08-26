pub mod embed;
pub mod sticker;
pub mod message_components;

use serde::{ Serialize, Deserialize };
use serde_json::{json, Value};
use error::{Error, ModelError, Result};
use crate::manager::http::HttpRessource;
use crate::models::Snowflake;
use crate::models::user::User;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Color(pub u64);

impl Color {
    pub fn from_hex(hex: impl Into<String>) -> Self {
        Self(u64::from_str_radix(&hex.into().replace('#', ""), 16).unwrap_or(0))
    }
}

impl HttpRessource for Color {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(color) => Ok(Self(color)),
            None => Err(Error::Model(ModelError::InvalidPayload("Failed to parse color".into())))
        }
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

impl HttpRessource for Emoji {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = Snowflake::from_raw(raw["id"].clone(), shard)?;
        let name = if let Some(name) = raw["name"].as_str() { name.to_string() } else { return Err(Error::Model(ModelError::InvalidPayload("Failed to parse emoji name".into()))) };
        let roles = if let Some(roles) = raw["roles"].as_array() {
                roles.iter().map(|role| Snowflake::from_raw(role.clone(), shard)).collect::<Result<Vec<Snowflake>>>()?
            } else {
                return Err(Error::Model(ModelError::InvalidPayload("Failed to parse emoji roles".into())))
            };
        let user = if let Some(users) = raw.get("users") {
                Some(User::from_raw(users.clone(), shard)?)
            } else {
                None
            };
        let require_colons = raw["require_colons"].as_bool();
        let managed = raw["managed"].as_bool();
        let animated = raw["animated"].as_bool();
        let available = raw["available"].as_bool();

        Ok(Self {
            id: Some(id),
            name,
            roles,
            user,
            require_colons,
            managed,
            animated,
            available,
        })
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