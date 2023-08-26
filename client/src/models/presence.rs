use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use error::Result;
use crate::manager::shard::Shard;

/// Contain all informations about a presence
///
/// Reference:
/// - [Discord API - Presence Structure](https://discord.com/developers/docs/topics/gateway#presence-update-presence-structure)
pub struct Presence {
    pub since: Option<u64>,
    pub activities: Vec<Activity>,
    pub status: StatusType,
    pub afk: bool
}

impl Presence {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "activities": self.activities,
            "status": self.status.to_string(),
            "afk": self.afk,
            "since": self.since
        })
    }
}

/// Contain all possible status types that can be used
///
/// Reference:
/// - [Discord API - Status Types](https://discord.com/developers/docs/topics/gateway#update-status-status-types)
pub enum StatusType {
    Online,
    Dnd,
    Idle,
    Invisible,
    Offline
}

impl ToString for StatusType {
    fn to_string(&self) -> String {
        match self {
            StatusType::Online => "online",
            StatusType::Dnd => "dnd",
            StatusType::Idle => "idle",
            StatusType::Invisible => "invisible",
            StatusType::Offline => "offline"
        }.to_string()
    }
}

/// Contain all informations about an activity
///
/// Reference:
/// - [Discord API - Activity Structure](https://discord.com/developers/docs/topics/gateway#activity-object-activity-structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub name: String,
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    pub url: Option<String>,
    /// DON'T SEND IT TO THE API
    ///
    /// This field i used while sending the presence to the API
    pub presence: String,
}

impl Activity {
    pub async fn set_presence(&self, shard: &Shard) -> Result<()> {
        let presence = Presence {
            since: None,
            activities: vec![self.clone()],
            status: StatusType::Online,
            afk: false
        };

        shard.set_presence(presence).await
    }
}

/// Contain all possible activity types that can be used
///
/// Reference:
/// - [Discord API - Activity Types](https://discord.com/developers/docs/topics/gateway#activity-object-activity-types)
#[derive(Debug, Clone)]
pub enum ActivityType {
    Game = 0,
    Streaming = 1,
    Listening = 2,
    Watching = 3,
    Custom = 4,
    Competing = 5
}
// impl in ActivityType to, with serde, get a ActivityType from a u8

impl Serialize for ActivityType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_u8(self.clone() as u8)
    }
}

impl<'de> Deserialize<'de> for ActivityType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let value = u8::deserialize(deserializer)?;

        match value {
            0 => Ok(ActivityType::Game),
            1 => Ok(ActivityType::Streaming),
            2 => Ok(ActivityType::Listening),
            3 => Ok(ActivityType::Watching),
            4 => Ok(ActivityType::Custom),
            5 => Ok(ActivityType::Competing),
            _ => Err(serde::de::Error::custom("Invalid activity type"))
        }
    }
}