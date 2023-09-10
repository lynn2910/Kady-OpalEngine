#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use chrono::{TimeZone, Utc};
use serde::{ Serialize, Deserialize };
use serde_json::Value;
use error::{ModelError, Result};
use error::Error::Model;
use crate::constants::MAX_MESSAGE_CACHE_SIZE;
use crate::manager::cache::{CacheDock, UpdateCache};
use crate::manager::http::{ApiResult, Http, HttpRessource};
use crate::models::guild::GuildId;
use crate::models::message::{Message, MessageBuilder};
use crate::models::Snowflake;




/// Represents a channel ID.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ChannelId(pub Snowflake);

impl Display for ChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ChannelId {
    pub async fn send_message(
        &self,
        http: &Http,
        payload: MessageBuilder
    ) -> Result<ApiResult<Message>> {
        http.send_message(self, payload).await
    }
}

impl From<String> for ChannelId {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<&str> for ChannelId {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl HttpRessource for ChannelId {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        Ok(Self(Snowflake::from_raw(raw, shard)?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Channel {
    Unknown(UnknownChannel),
    GuildText(GuildTextChannel),
    Dm(Dm),
    GuildVoice(GuildVoice),
    GuildCategory(GuildCategory),
    GuildAnnouncement(GuildAnnouncement),
    GuildStageVoice(GuildStageVoice),
    GuildForum(GuildForum),
    Thread(Thread),
}

impl UpdateCache for Channel {
    fn update(&mut self, from: &Self) {
        match self {
            Self::Unknown(c) => {
                if let Self::Unknown(from) = from {
                    c.update(from);
                }
            },
            Self::GuildText(c) => {
                if let Self::GuildText(from) = from {
                    c.update(from);
                }
            },
            Self::Dm(c) => {
                if let Self::Dm(from) = from {
                    c.update(from);
                }
            },
            Self::GuildVoice(c) => {
                if let Self::GuildVoice(from) = from {
                    c.update(from);
                }
            },
            Self::GuildCategory(c) => {
                if let Self::GuildCategory(from) = from {
                    c.update(from);
                }
            },
            Self::GuildAnnouncement(c) => {
                if let Self::GuildAnnouncement(from) = from {
                    c.update(from);
                }
            },
            Self::GuildStageVoice(c) => {
                if let Self::GuildStageVoice(from) = from {
                    c.update(from);
                }
            },
            Self::GuildForum(c) => {
                if let Self::GuildForum(from) = from {
                    c.update(from);
                }
            },
            Self::Thread(c) => {
                if let Self::Thread(from) = from {
                    c.update(from);
                }
            }
        }
    }
}

impl HttpRessource for Channel {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let kind = ChannelKind::from_raw(raw["type"].to_owned(), shard)?;

        match kind {
            ChannelKind::GuildText => Ok(Self::GuildText(GuildTextChannel::from_raw(raw, shard)?)),
            ChannelKind::Dm => Ok(Self::Dm(Dm::from_raw(raw, shard)?)),
            ChannelKind::GuildVoice => Ok(Self::GuildVoice(GuildVoice::from_raw(raw, shard)?)),
            ChannelKind::GuildCategory => Ok(Self::GuildCategory(GuildCategory::from_raw(raw, shard)?)),
            ChannelKind::GuildAnnouncement => Ok(Self::GuildAnnouncement(GuildAnnouncement::from_raw(raw, shard)?)),
            ChannelKind::GuildStageVoice => Ok(Self::GuildStageVoice(GuildStageVoice::from_raw(raw, shard)?)),
            ChannelKind::GuildForum => Ok(Self::GuildForum(GuildForum::from_raw(raw, shard)?)),
            ChannelKind::PublicThread => Ok(Self::Thread(Thread::PublicThread(PublicThread::from_raw(raw, shard)?))),
            ChannelKind::PrivateThread => Ok(Self::Thread(Thread::PrivateThread(PrivateThread::from_raw(raw, shard)?))),
            ChannelKind::AnnouncementThread => Ok(Self::Thread(Thread::PublicThread(PublicThread::from_raw(raw, shard)?))),
            ChannelKind::GuildDirectory | ChannelKind::GroupDm => Ok(Self::Unknown(UnknownChannel::from_raw(raw, shard)?)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Thread {
    PublicThread(PublicThread),
    PrivateThread(PrivateThread),
    AnnouncementThread(AnnouncementThread),
}

impl UpdateCache for Thread {
    fn update(&mut self, from: &Self) {
        match self {
            Self::AnnouncementThread(c) => {
                if let Self::AnnouncementThread(from) = from {
                    c.update(from);
                }
            },
            Self::PrivateThread(c) => {
                if let Self::PrivateThread(from) = from {
                    c.update(from);
                }
            },
            Self::PublicThread(c) => {
                if let Self::PublicThread(from) = from {
                    c.update(from);
                }
            }
        }
    }
}

/// Represents a channel which is not known to the client.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct UnknownChannel {
    pub id: ChannelId,
    pub name: String,
    pub kind: ChannelKind,
    pub guild_id: Option<GuildId>,
}

impl UpdateCache for UnknownChannel {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.kind != from.kind {
            self.kind = from.kind.clone();
        }
        if self.guild_id != from.guild_id {
            self.guild_id = from.guild_id.clone();
        }
    }
}

impl HttpRessource for UnknownChannel {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw["id"].to_owned(), shard)?;
        let name = raw["name"].as_str().map(|s| s.to_string()).unwrap_or_else(|| "Unknown".to_string());
        let kind = ChannelKind::from_raw(raw["type"].to_owned(), shard)?;
        let guild_id = raw["guild_id"].as_str().map(|s| GuildId(s.to_string().into()));

        Ok(Self {
            id,
            name,
            kind,
            guild_id,
        })
    }
}

/// Represents a guild text channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildTextChannel {
    pub id: ChannelId,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub guild_id: GuildId,
    pub position: Option<u64>,
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    pub topic: Option<String>,
    pub nsfw: Option<bool>,
    /// The id of the last message sent in this channel (may not point to an existing or valid message)
    pub last_message_id: Option<String>,
    /// Amount of seconds a user has to wait before sending another message (0-21600)
    pub rate_limit_per_user: Option<u64>,
    pub parent_id: Option<ChannelId>,
    pub last_pin_timestamp: Option<chrono::DateTime<Utc>>,
    /// Contain the permissions for the user in the channel, including overwrites, only when part of the interaction object
    pub permissions: Option<String>,
    pub flags: Option<u64>,

    pub messages: CacheDock<Snowflake, Message>
}

impl UpdateCache for GuildTextChannel {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.icon != from.icon {
            self.icon = from.icon.clone();
        }
        if self.guild_id != from.guild_id {
            self.guild_id = from.guild_id.clone();
        }
        if self.position != from.position {
            self.position = from.position;
        }
        if self.permission_overwrites != from.permission_overwrites {
            self.permission_overwrites = from.permission_overwrites.clone();
        }
        if self.topic != from.topic {
            self.topic = from.topic.clone();
        }
        if self.nsfw != from.nsfw {
            self.nsfw = from.nsfw;
        }
        if self.last_message_id != from.last_message_id {
            self.last_message_id = from.last_message_id.clone();
        }
        if self.rate_limit_per_user != from.rate_limit_per_user {
            self.rate_limit_per_user = from.rate_limit_per_user;
        }
        if self.parent_id != from.parent_id {
            self.parent_id = from.parent_id.clone();
        }
        if self.last_pin_timestamp != from.last_pin_timestamp {
            self.last_pin_timestamp = from.last_pin_timestamp;
        }
        if self.permissions != from.permissions {
            self.permissions = from.permissions.clone();
        }
        if self.flags != from.flags {
            self.flags = from.flags;
        }
    }
}

impl HttpRessource for GuildTextChannel {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw["id"].to_owned(), shard)?;
        let name = raw["name"].as_str().map(|s| s.to_string());
        let icon = raw["icon"].as_str().map(|s| s.to_string());
        let guild_id = GuildId::from_raw(raw["guild_id"].to_owned(), shard)?;
        let position = raw["position"].as_u64();
        let permission_overwrites = raw["permission_overwrites"].as_array().map(|a| {
            a.iter().map(|v| PermissionOverwrite::from_raw(v.to_owned(), shard)).collect::<Result<Vec<_>>>()
        }).transpose()?;
        let topic = raw["topic"].as_str().map(|s| s.to_string());
        let nsfw = raw["nsfw"].as_bool();
        let last_message_id = raw["last_message_id"].as_str().map(|s| s.to_string());
        let rate_limit_per_user = raw["rate_limit_per_user"].as_u64();

        let parent_id = if let Some(s) = raw.get("parent_id") {
            Some(ChannelId::from_raw(s.to_owned(), shard)?)
        } else {
            None
        };

        let last_pin_timestamp = if let Some(ts) = raw["last_pin_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => Some(t.with_timezone(&Utc)),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            None
        };

        let permissions = raw["permissions"].as_str().map(|s| s.to_string());
        let flags = raw["flags"].as_u64();

        Ok(Self {
            id,
            name,
            icon,
            guild_id,
            position,
            permission_overwrites,
            topic,
            nsfw,
            last_message_id,
            rate_limit_per_user,
            parent_id,
            last_pin_timestamp,
            permissions,
            flags,
            messages: CacheDock::new(MAX_MESSAGE_CACHE_SIZE)
        })
    }
}

/// Represents a DM channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Dm {
    pub id: ChannelId,
    pub kind: ChannelKind,
    pub last_message_id: Option<String>,
    pub icon: Option<String>,
    pub last_pin_timestamp: Option<chrono::DateTime<Utc>>,
    /// Contain the permissions for the user in the channel, including overwrites, only when part of the interaction object
    pub permissions: Option<String>,

    pub messages: CacheDock<Snowflake, Message>
}

impl UpdateCache for Dm {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.kind != from.kind {
            self.kind = from.kind.clone();
        }
        if self.last_message_id != from.last_message_id {
            self.last_message_id = from.last_message_id.clone();
        }
        if self.icon != from.icon {
            self.icon = from.icon.clone();
        }
        if self.last_pin_timestamp != from.last_pin_timestamp {
            self.last_pin_timestamp = from.last_pin_timestamp;
        }
        if self.permissions != from.permissions {
            self.permissions = from.permissions.clone();
        }
    }
}

impl HttpRessource for Dm {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw["id"].to_owned(), shard)?;
        let kind = ChannelKind::from_raw(raw["type"].to_owned(), shard)?;
        let last_message_id = raw["last_message_id"].as_str().map(|s| s.to_string());
        let icon = raw["icon"].as_str().map(|s| s.to_string());
        let last_pin_timestamp = if let Some(ts) = raw["last_pin_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => Some(t.with_timezone(&Utc)),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            None
        };
        let permissions = raw["permissions"].as_str().map(|s| s.to_string());

        Ok(Self {
            id,
            kind,
            last_message_id,
            icon,
            last_pin_timestamp,
            permissions,
            messages: CacheDock::new(MAX_MESSAGE_CACHE_SIZE)
        })
    }
}

/// Represents a guild voice channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildVoice {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: GuildId,
    pub position: Option<u64>,
    /// The permission overwrites for this voice channel
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    pub bitrate: Option<u64>,
    pub user_limit: Option<u64>,
    pub parent_id: Option<ChannelId>,
    pub rtc_region: Option<String>,
    pub video_quality_mode: Option<u64>,
    /// Contain the permissions for the user in the channel, including overwrites, only when part of the interaction object
    pub permissions: Option<String>,
    pub nsfw: Option<bool>
}

impl UpdateCache for GuildVoice {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.guild_id != from.guild_id {
            self.guild_id = from.guild_id.clone();
        }
        if self.position != from.position {
            self.position = from.position;
        }
        if self.permission_overwrites != from.permission_overwrites {
            self.permission_overwrites = from.permission_overwrites.clone();
        }
        if self.bitrate != from.bitrate {
            self.bitrate = from.bitrate;
        }
        if self.user_limit != from.user_limit {
            self.user_limit = from.user_limit;
        }
        if self.parent_id != from.parent_id {
            self.parent_id = from.parent_id.clone();
        }
        if self.rtc_region != from.rtc_region {
            self.rtc_region = from.rtc_region.clone();
        }
        if self.video_quality_mode != from.video_quality_mode {
            self.video_quality_mode = from.video_quality_mode;
        }
        if self.permissions != from.permissions {
            self.permissions = from.permissions.clone();
        }
        if self.nsfw != from.nsfw {
            self.nsfw = from.nsfw;
        }
    }
}

impl HttpRessource for GuildVoice {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw["id"].to_owned(), shard)?;
        let name = raw["name"].as_str().map(|s| s.to_string());
        let guild_id = GuildId::from_raw(raw["guild_id"].to_owned(), shard)?;
        let position = raw["position"].as_u64();
        let permission_overwrites = raw["permission_overwrites"].as_array().map(|a| {
            a.iter().map(|v| PermissionOverwrite::from_raw(v.to_owned(), shard)).collect::<Result<Vec<_>>>()
        }).transpose()?;
        let bitrate = raw["bitrate"].as_u64();
        let user_limit = raw["user_limit"].as_u64();
        let parent_id = raw["parent_id"].as_str().map(|s| ChannelId(s.to_string().into()));
        let rtc_region = raw["rtc_region"].as_str().map(|s| s.to_string());
        let video_quality_mode = raw["video_quality_mode"].as_u64();
        let permissions = raw["permissions"].as_str().map(|s| s.to_string());
        let nsfw = raw["nsfw"].as_bool();

        Ok(Self {
            id,
            name,
            guild_id,
            position,
            permission_overwrites,
            bitrate,
            user_limit,
            parent_id,
            rtc_region,
            video_quality_mode,
            permissions,
            nsfw
        })
    }
}

/// Represents a guild category channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildCategory {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: GuildId,
    pub position: Option<u64>,
    /// The permission overwrites for this category
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    /// Contain the permissions for the user in the channel, including overwrites, only when part of the interaction object
    pub permissions: Option<String>,
}

impl UpdateCache for GuildCategory {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.guild_id != from.guild_id {
            self.guild_id = from.guild_id.clone();
        }
        if self.position != from.position {
            self.position = from.position;
        }
        if self.permission_overwrites != from.permission_overwrites {
            self.permission_overwrites = from.permission_overwrites.clone();
        }
        if self.permissions != from.permissions {
            self.permissions = from.permissions.clone();
        }
    }
}

impl HttpRessource for GuildCategory {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw["id"].to_owned(), shard)?;
        let name = raw["name"].as_str().map(|s| s.to_string());
        let guild_id = GuildId::from_raw(raw["guild_id"].to_owned(), shard)?;
        let position = raw["position"].as_u64();
        let permission_overwrites = raw["permission_overwrites"].as_array().map(|a| {
            a.iter().map(|v| PermissionOverwrite::from_raw(v.to_owned(), shard)).collect::<Result<Vec<_>>>()
        }).transpose()?;
        let permissions = raw["permissions"].as_str().map(|s| s.to_string());

        Ok(Self {
            id,
            name,
            guild_id,
            position,
            permission_overwrites,
            permissions
        })
    }
}


/// Represents a guild announcement channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildAnnouncement {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: GuildId,
    pub position: Option<u64>,
    /// The permission overwrites for this announcement channel
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    pub nsfw: Option<bool>,
    /// The id of the last message sent in this channel (may not point to an existing or valid message)
    pub last_message_id: Option<String>,
    /// Amount of seconds a user has to wait before sending another message (0-21600)
    pub rate_limit_per_user: Option<u64>,
    pub parent_id: Option<ChannelId>,
    pub last_pin_timestamp: Option<chrono::DateTime<Utc>>,
    /// Contain the permissions for the user in the channel, including overwrites, only when part of the interaction object
    pub permissions: Option<String>,

    pub messages: CacheDock<Snowflake, Message>
}

impl UpdateCache for GuildAnnouncement {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.guild_id != from.guild_id {
            self.guild_id = from.guild_id.clone();
        }
        if self.position != from.position {
            self.position = from.position;
        }
        if self.permission_overwrites != from.permission_overwrites {
            self.permission_overwrites = from.permission_overwrites.clone();
        }
        if self.nsfw != from.nsfw {
            self.nsfw = from.nsfw;
        }
        if self.last_message_id != from.last_message_id {
            self.last_message_id = from.last_message_id.clone();
        }
        if self.rate_limit_per_user != from.rate_limit_per_user {
            self.rate_limit_per_user = from.rate_limit_per_user;
        }
        if self.parent_id != from.parent_id {
            self.parent_id = from.parent_id.clone();
        }
        if self.last_pin_timestamp != from.last_pin_timestamp {
            self.last_pin_timestamp = from.last_pin_timestamp;
        }
        if self.permissions != from.permissions {
            self.permissions = from.permissions.clone();
        }
    }
}

impl HttpRessource for GuildAnnouncement {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw.get("id").ok_or(Model(ModelError::MissingField("id".into())))?.clone(), shard)?;
        let name = raw.get("name").and_then(|x| x.as_str()).map(|x| x.to_string());
        let guild_id = GuildId::from_raw(raw.get("guild_id").ok_or(Model(ModelError::MissingField("guild_id".into())))?.clone(), shard)?;
        let position = raw.get("position").and_then(|x| x.as_u64());
        let permission_overwrites = raw.get("permission_overwrites").and_then(|x| x.as_array()).map(|x| x.iter().map(|x| PermissionOverwrite::from_raw(x.clone(), shard)).collect::<Result<Vec<_>>>()).transpose()?;
        let nsfw = raw.get("nsfw").and_then(|x| x.as_bool());
        let last_message_id = raw.get("last_message_id").and_then(|x| x.as_str()).map(|x| x.to_string());
        let rate_limit_per_user = raw.get("rate_limit_per_user").and_then(|x| x.as_u64());
        let parent_id = raw.get("parent_id").map(|x| ChannelId::from_raw(x.clone(), shard)).transpose()?;
        let last_pin_timestamp = if let Some(ts) = raw["last_pin_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => Some(t.with_timezone(&Utc)),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            None
        };
        let permissions = raw.get("permissions").and_then(|x| x.as_str()).map(|x| x.to_string());

        Ok(Self {
            id,
            name,
            guild_id,
            position,
            permission_overwrites,
            nsfw,
            last_message_id,
            rate_limit_per_user,
            parent_id,
            last_pin_timestamp,
            permissions,
            messages: CacheDock::new(MAX_MESSAGE_CACHE_SIZE)
        })

    }
}

/// Represents an announcement thread channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AnnouncementThread {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: GuildId,
    pub parent_id: Option<ChannelId>,
    pub creator_id: String,
    pub kind: ChannelKind,
    pub last_message_id: Option<String>,
    pub locked: bool,
    pub auto_archive_duration: u64,
    pub archive_timestamp: chrono::DateTime<Utc>,
    pub locked_timestamp: Option<chrono::DateTime<Utc>>,
    pub message_count: u64,
    pub member_count: u64,
    pub thread_metadata: ThreadMetadata,
    pub default_auto_archive_duration: u64,

    pub messages: CacheDock<Snowflake, Message>
}

impl UpdateCache for AnnouncementThread {
    fn update(&mut self, from: &Self) {
        if self.id != from.id { self.id = from.id.clone(); }
        if self.name != from.name { self.name = from.name.clone(); }
        if self.guild_id != from.guild_id { self.guild_id = from.guild_id.clone(); }
        if self.parent_id != from.parent_id { self.parent_id = from.parent_id.clone(); }
        if self.creator_id != from.creator_id { self.creator_id = from.creator_id.clone(); }
        if self.kind != from.kind { self.kind = from.kind.clone(); }
        if self.last_message_id != from.last_message_id { self.last_message_id = from.last_message_id.clone(); }
        if self.locked != from.locked { self.locked = from.locked; }
        if self.auto_archive_duration != from.auto_archive_duration { self.auto_archive_duration = from.auto_archive_duration; }
        if self.archive_timestamp != from.archive_timestamp { self.archive_timestamp = from.archive_timestamp; }
        if self.locked_timestamp != from.locked_timestamp { self.locked_timestamp = from.locked_timestamp; }
        if self.message_count != from.message_count { self.message_count = from.message_count; }
        if self.member_count != from.member_count { self.member_count = from.member_count; }
        if self.thread_metadata != from.thread_metadata { self.thread_metadata = from.thread_metadata.clone(); }
        if self.default_auto_archive_duration != from.default_auto_archive_duration { self.default_auto_archive_duration = from.default_auto_archive_duration; }
    }
}

impl HttpRessource for AnnouncementThread {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw.get("id").ok_or(Model(ModelError::MissingField("id".into())))?.clone(), shard)?;
        let name = raw.get("name").and_then(|x| x.as_str()).map(|x| x.to_owned());
        let guild_id = GuildId::from_raw(raw.get("guild_id").ok_or(Model(ModelError::MissingField("guild_id".into())))?.clone(), shard)?;
        let parent_id = raw.get("parent_id").map(|x| ChannelId::from_raw(x.to_owned(), shard)).transpose()?;
        let creator_id = raw.get("creator_id").ok_or(Model(ModelError::MissingField("creator_id".into())))?.as_str().ok_or(Model(ModelError::InvalidPayload("creator_id".into())))?.to_owned();
        let kind = ChannelKind::from_raw(raw["type"].clone(), shard)?;
        let last_message_id = raw.get("last_message_id").and_then(|x| x.as_str()).map(|x| x.to_owned());
        let locked = raw.get("locked").ok_or(Model(ModelError::MissingField("locked".into())))?.as_bool().ok_or(Model(ModelError::InvalidPayload("locked".into())))?;
        let auto_archive_duration = raw.get("auto_archive_duration").ok_or(Model(ModelError::MissingField("auto_archive_duration".into())))?.as_u64().ok_or(Model(ModelError::InvalidPayload("auto_archive_duration".into())))?;
        let archive_timestamp = if let Some(ts) = raw["archive_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => t.with_timezone(&Utc),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            Utc::now()
        };
        let locked_timestamp = if let Some(ts) = raw["locked_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => Some(t.with_timezone(&Utc)),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            None
        };
        let message_count = raw.get("message_count").ok_or(Model(ModelError::MissingField("message_count".into())))?.as_u64().ok_or(Model(ModelError::InvalidPayload("message_count".into())))?;
        let member_count = raw.get("member_count").ok_or(Model(ModelError::MissingField("member_count".into())))?.as_u64().ok_or(Model(ModelError::InvalidPayload("member_count".into())))?;
        let thread_metadata = ThreadMetadata::from_raw(raw.get("thread_metadata").ok_or(Model(ModelError::MissingField("thread_metadata".into())))?.to_owned(), shard)?;
        let default_auto_archive_duration = raw.get("default_auto_archive_duration").ok_or(Model(ModelError::MissingField("default_auto_archive_duration".into())))?.as_u64().ok_or(Model(ModelError::InvalidPayload("default_auto_archive_duration".into())))?;

        Ok(Self {
            id,
            name,
            guild_id,
            parent_id,
            creator_id,
            kind,
            last_message_id,
            locked,
            auto_archive_duration,
            archive_timestamp,
            locked_timestamp,
            message_count,
            member_count,
            thread_metadata,
            default_auto_archive_duration,
            messages: CacheDock::new(MAX_MESSAGE_CACHE_SIZE)
        })
    }
}

/// Represents a public thread channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PublicThread {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: GuildId,
    pub parent_id: Option<ChannelId>,
    pub creator_id: String,
    pub kind: ChannelKind,
    pub last_message_id: Option<String>,
    pub locked: bool,
    pub auto_archive_duration: u64,
    pub archive_timestamp: chrono::DateTime<Utc>,
    pub locked_timestamp: Option<chrono::DateTime<Utc>>,
    pub message_count: u64,
    pub member_count: u64,
    pub thread_metadata: ThreadMetadata,
    pub default_auto_archive_duration: u64,

    pub messages: CacheDock<Snowflake, Message>
}

impl UpdateCache for PublicThread {
    fn update(&mut self, from: &Self) {
        if self.id != from.id { self.id = from.id.clone(); }
        if self.name != from.name { self.name = from.name.clone(); }
        if self.guild_id != from.guild_id { self.guild_id = from.guild_id.clone(); }
        if self.parent_id != from.parent_id { self.parent_id = from.parent_id.clone(); }
        if self.creator_id != from.creator_id { self.creator_id = from.creator_id.clone(); }
        if self.kind != from.kind { self.kind = from.kind.clone(); }
        if self.last_message_id != from.last_message_id { self.last_message_id = from.last_message_id.clone(); }
        if self.locked != from.locked { self.locked = from.locked; }
        if self.auto_archive_duration != from.auto_archive_duration { self.auto_archive_duration = from.auto_archive_duration; }
        if self.archive_timestamp != from.archive_timestamp { self.archive_timestamp = from.archive_timestamp; }
        if self.locked_timestamp != from.locked_timestamp { self.locked_timestamp = from.locked_timestamp; }
        if self.message_count != from.message_count { self.message_count = from.message_count; }
        if self.member_count != from.member_count { self.member_count = from.member_count; }
        if self.thread_metadata != from.thread_metadata { self.thread_metadata = from.thread_metadata.clone(); }
        if self.default_auto_archive_duration != from.default_auto_archive_duration { self.default_auto_archive_duration = from.default_auto_archive_duration; }
    }
}

impl HttpRessource for PublicThread {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw.get("id").ok_or(Model(ModelError::MissingField("id".into())))?.clone(), shard)?;
        let name = raw.get("name").and_then(|x| x.as_str()).map(|x| x.to_string());
        let guild_id = GuildId::from_raw(raw.get("guild_id").ok_or(Model(ModelError::MissingField("guild_d".into())))?.clone(), shard)?;
        let parent_id = raw.get("parent_id").map(|x| ChannelId::from_raw(x.clone(), shard)).transpose()?;
        let creator_id = raw.get("creator_id").and_then(|x| x.as_str()).ok_or(Model(ModelError::MissingField("creator_id".into())))?.to_string();
        let kind = ChannelKind::from_raw(raw.get("type").ok_or(Model(ModelError::MissingField("type".into())))?.clone(), shard)?;
        let last_message_id = raw.get("last_message_id").and_then(|x| x.as_str()).map(|x| x.to_string());
        let locked = raw.get("locked").and_then(|x| x.as_bool()).ok_or(Model(ModelError::MissingField("locked".into())))?;
        let auto_archive_duration = raw.get("auto_archive_duration").and_then(|x| x.as_u64()).ok_or(Model(ModelError::MissingField("auto_archive_duration".into())))?;
        let archive_timestamp = if let Some(ts) = raw["archive_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => t.with_timezone(&Utc),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            Utc::now()
        };
        let locked_timestamp = raw.get("locked_timestamp").and_then(|x| x.as_str()).and_then(|x| x.parse::<chrono::DateTime<Utc>>().ok());
        let message_count = raw.get("message_count").and_then(|x| x.as_u64()).ok_or(Model(ModelError::MissingField("message_count".into())))?;
        let member_count = raw.get("member_count").and_then(|x| x.as_u64()).ok_or(Model(ModelError::MissingField("member_count".into())))?;
        let thread_metadata = ThreadMetadata::from_raw(raw.get("thread_metadata").ok_or(Model(ModelError::MissingField("thread_metadata".into())))?.clone(), shard)?;
        let default_auto_archive_duration = raw.get("default_auto_archive_duration").and_then(|x| x.as_u64()).ok_or(Model(ModelError::MissingField("default_auto_archive_duration".into())))?;

        Ok(Self {
            id,
            name,
            guild_id,
            parent_id,
            creator_id,
            kind,
            last_message_id,
            locked,
            auto_archive_duration,
            archive_timestamp,
            locked_timestamp,
            message_count,
            member_count,
            thread_metadata,
            default_auto_archive_duration,
            messages: CacheDock::new(MAX_MESSAGE_CACHE_SIZE)
        })
    }
}

/// Represents a private thread channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PrivateThread {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: GuildId,
    pub parent_id: Option<ChannelId>,
    pub creator_id: String,
    pub kind: ChannelKind,
    pub last_message_id: Option<String>,
    pub locked: bool,
    pub auto_archive_duration: u64,
    pub archive_timestamp: chrono::DateTime<Utc>,
    pub locked_timestamp: Option<chrono::DateTime<Utc>>,
    pub message_count: u64,
    pub member_count: u64,
    pub thread_metadata: ThreadMetadata,
    pub default_auto_archive_duration: u64,

    pub messages: CacheDock<Snowflake, Message>
}

impl UpdateCache for PrivateThread {
    fn update(&mut self, from: &Self) {
        if self.id != from.id { self.id = from.id.clone(); }
        if self.name != from.name { self.name = from.name.clone(); }
        if self.guild_id != from.guild_id { self.guild_id = from.guild_id.clone(); }
        if self.parent_id != from.parent_id { self.parent_id = from.parent_id.clone(); }
        if self.creator_id != from.creator_id { self.creator_id = from.creator_id.clone(); }
        if self.kind != from.kind { self.kind = from.kind.clone(); }
        if self.last_message_id != from.last_message_id { self.last_message_id = from.last_message_id.clone(); }
        if self.locked != from.locked { self.locked = from.locked; }
        if self.auto_archive_duration != from.auto_archive_duration { self.auto_archive_duration = from.auto_archive_duration; }
        if self.archive_timestamp != from.archive_timestamp { self.archive_timestamp = from.archive_timestamp; }
        if self.locked_timestamp != from.locked_timestamp { self.locked_timestamp = from.locked_timestamp; }
        if self.message_count != from.message_count { self.message_count = from.message_count; }
        if self.member_count != from.member_count { self.member_count = from.member_count; }
        if self.thread_metadata != from.thread_metadata { self.thread_metadata = from.thread_metadata.clone(); }
        if self.default_auto_archive_duration != from.default_auto_archive_duration { self.default_auto_archive_duration = from.default_auto_archive_duration; }
    }
}

impl HttpRessource for PrivateThread {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw.get("id").ok_or(Model(ModelError::MissingField("id".into())))?.clone(), shard)?;
        let name = raw.get("name").and_then(|x| x.as_str()).map(|x| x.to_string());
        let guild_id = GuildId::from_raw(raw.get("guild_id").ok_or(Model(ModelError::MissingField("guild_d".into())))?.clone(), shard)?;
        let parent_id = raw.get("parent_id").map(|x| ChannelId::from_raw(x.clone(), shard)).transpose()?;
        let creator_id = raw.get("creator_id").and_then(|x| x.as_str()).ok_or(Model(ModelError::MissingField("creator_id".into())))?.to_string();
        let kind = ChannelKind::from_raw(raw.get("type").ok_or(Model(ModelError::MissingField("type".into())))?.clone(), shard)?;
        let last_message_id = raw.get("last_message_id").and_then(|x| x.as_str()).map(|x| x.to_string());
        let locked = raw.get("locked").and_then(|x| x.as_bool()).ok_or(Model(ModelError::MissingField("locked".into())))?;
        let auto_archive_duration = raw.get("auto_archive_duration").and_then(|x| x.as_u64()).ok_or(Model(ModelError::MissingField("auto_archive_duration".into())))?;
        let archive_timestamp = if let Some(ts) = raw["archive_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => t.with_timezone(&Utc),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            Utc::now()
        };
        let locked_timestamp = raw.get("locked_timestamp").and_then(|x| x.as_str()).and_then(|x| x.parse::<chrono::DateTime<Utc>>().ok());
        let message_count = raw.get("message_count").and_then(|x| x.as_u64()).ok_or(Model(ModelError::MissingField("message_count".into())))?;
        let member_count = raw.get("member_count").and_then(|x| x.as_u64()).ok_or(Model(ModelError::MissingField("member_count".into())))?;
        let thread_metadata = ThreadMetadata::from_raw(raw.get("thread_metadata").ok_or(Model(ModelError::MissingField("thread_metadata".into())))?.clone(), shard)?;
        let default_auto_archive_duration = raw.get("default_auto_archive_duration").and_then(|x| x.as_u64()).ok_or(Model(ModelError::MissingField("default_auto_archive_duration".into())))?;

        Ok(Self {
            id,
            name,
            guild_id,
            parent_id,
            creator_id,
            kind,
            last_message_id,
            locked,
            auto_archive_duration,
            archive_timestamp,
            locked_timestamp,
            message_count,
            member_count,
            thread_metadata,
            default_auto_archive_duration,
            messages: CacheDock::new(MAX_MESSAGE_CACHE_SIZE)
        })
    }
}

/// Represents a guild stage voice channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildStageVoice {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: GuildId,
    pub position: Option<u64>,
    /// The permission overwrites for this stage voice channel
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    pub bitrate: Option<u64>,
    pub user_limit: Option<u64>,
    pub parent_id: Option<ChannelId>,
    pub rtc_region: Option<String>,
    pub video_quality_mode: Option<u64>,
    /// Contain the permissions for the user in the channel, including overwrites, only when part of the interaction object
    pub permissions: Option<String>,
    pub nsfw: Option<bool>,

    pub messages: CacheDock<Snowflake, Message>
}

impl UpdateCache for GuildStageVoice {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.guild_id != from.guild_id {
            self.guild_id = from.guild_id.clone();
        }
        if self.position != from.position {
            self.position = from.position;
        }
        if self.permission_overwrites != from.permission_overwrites {
            self.permission_overwrites = from.permission_overwrites.clone();
        }
        if self.bitrate != from.bitrate {
            self.bitrate = from.bitrate;
        }
        if self.user_limit != from.user_limit {
            self.user_limit = from.user_limit;
        }
        if self.parent_id != from.parent_id {
            self.parent_id = from.parent_id.clone();
        }
        if self.rtc_region != from.rtc_region {
            self.rtc_region = from.rtc_region.clone();
        }
        if self.video_quality_mode != from.video_quality_mode {
            self.video_quality_mode = from.video_quality_mode;
        }
        if self.permissions != from.permissions {
            self.permissions = from.permissions.clone();
        }
        if self.nsfw != from.nsfw {
            self.nsfw = from.nsfw;
        }
    }
}

impl HttpRessource for GuildStageVoice {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw["id"].clone(), shard)?;
        let name = raw["name"].as_str().map(|x| x.to_string());
        let guild_id = GuildId::from_raw(raw["guild_id"].clone(), shard)?;
        let position = raw["position"].as_u64();
        let permission_overwrites = raw["permission_overwrites"].as_array().map(|x| x.iter().map(|x| PermissionOverwrite::from_raw(x.clone(), shard)).collect::<Result<Vec<_>>>()).transpose()?;
        let bitrate = raw["bitrate"].as_u64();
        let user_limit = raw["user_limit"].as_u64();
        let parent_id = raw.get("parent_id").map(|x| ChannelId::from_raw(x.to_owned(), shard)).transpose()?;
        let rtc_region = raw["rtc_region"].as_str().map(|x| x.to_string());
        let video_quality_mode = raw["video_quality_mode"].as_u64();
        let permissions = raw["permissions"].as_str().map(|x| x.to_string());
        let nsfw = raw["nsfw"].as_bool();

        Ok(Self {
            id,
            name,
            guild_id,
            position,
            permission_overwrites,
            bitrate,
            user_limit,
            parent_id,
            rtc_region,
            video_quality_mode,
            permissions,
            nsfw,
            messages: CacheDock::new(MAX_MESSAGE_CACHE_SIZE)
        })
    }
}

/// Represents a guild forum channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildForum {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: GuildId,
    pub position: Option<u64>,
    /// The permission overwrites for this forum channel
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    pub nsfw: Option<bool>,
    /// The id of the last message sent in this channel (may not point to an existing or valid message)
    pub last_message_id: Option<String>,
    /// Amount of seconds a user has to wait before sending another message (0-21600)
    pub rate_limit_per_user: Option<u64>,
    pub parent_id: Option<ChannelId>,
    pub last_pin_timestamp: Option<chrono::DateTime<Utc>>,
    /// Contain the permissions for the user in the channel, including overwrites, only when part of the interaction object
    pub permissions: Option<String>,
    pub messages: CacheDock<Snowflake, Message>
}

impl UpdateCache for GuildForum {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.guild_id != from.guild_id {
            self.guild_id = from.guild_id.clone();
        }
        if self.position != from.position {
            self.position = from.position;
        }
        if self.permission_overwrites != from.permission_overwrites {
            self.permission_overwrites = from.permission_overwrites.clone();
        }
        if self.nsfw != from.nsfw {
            self.nsfw = from.nsfw;
        }
        if self.last_message_id != from.last_message_id {
            self.last_message_id = from.last_message_id.clone();
        }
        if self.rate_limit_per_user != from.rate_limit_per_user {
            self.rate_limit_per_user = from.rate_limit_per_user;
        }
        if self.parent_id != from.parent_id {
            self.parent_id = from.parent_id.clone();
        }
        if self.last_pin_timestamp != from.last_pin_timestamp {
            self.last_pin_timestamp = from.last_pin_timestamp;
        }
        if self.permissions != from.permissions {
            self.permissions = from.permissions.clone();
        }
    }
}

impl HttpRessource for GuildForum {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = ChannelId::from_raw(raw["id"].clone(), shard)?;
        let name = raw["name"].as_str().map(|x| x.to_owned());
        let guild_id = GuildId::from_raw(raw["guild_id"].clone(), shard)?;
        let position = raw["position"].as_u64();
        let permission_overwrites = raw["permission_overwrites"].as_array().map(|x| {
            x.iter().map(|x| PermissionOverwrite::from_raw(x.clone(), shard)).collect::<Result<Vec<_>>>()
        }).transpose()?;
        let nsfw = raw["nsfw"].as_bool();
        let last_message_id = raw["last_message_id"].as_str().map(|x| x.to_owned());
        let rate_limit_per_user = raw["rate_limit_per_user"].as_u64();
        let parent_id = raw.get("parent_id").map(|x| ChannelId::from_raw(x.to_owned(), shard).unwrap());
        let last_pin_timestamp = if let Some(ts) = raw["last_pin_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => Some(t.with_timezone(&Utc)),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            None
        };
        let permissions = raw["permissions"].as_str().map(|x| x.to_owned());

        Ok(Self {
            id,
            name,
            guild_id,
            position,
            permission_overwrites,
            nsfw,
            last_message_id,
            rate_limit_per_user,
            parent_id,
            last_pin_timestamp,
            permissions,
            messages: CacheDock::new(MAX_MESSAGE_CACHE_SIZE)
        })
    }
}

/// Represent every kind of channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-types)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum ChannelKind {
    GuildText = 0,
    Dm = 1,
    GuildVoice = 2,
    /// Ignored
    GroupDm = 3,
    GuildCategory = 4,
    GuildAnnouncement = 5,
    AnnouncementThread = 10,
    PublicThread = 11,
    PrivateThread = 12,
    GuildStageVoice = 13,
    /// Ignored
    GuildDirectory = 14,
    GuildForum = 15
}

impl HttpRessource for ChannelKind {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(0) => Ok(ChannelKind::GuildText),
            Some(1) => Ok(ChannelKind::Dm),
            Some(2) => Ok(ChannelKind::GuildVoice),
            Some(3) => Ok(ChannelKind::GroupDm),
            Some(4) => Ok(ChannelKind::GuildCategory),
            Some(5) => Ok(ChannelKind::GuildAnnouncement),
            Some(10) => Ok(ChannelKind::AnnouncementThread),
            Some(11) => Ok(ChannelKind::PublicThread),
            Some(12) => Ok(ChannelKind::PrivateThread),
            Some(13) => Ok(ChannelKind::GuildStageVoice),
            Some(14) => Ok(ChannelKind::GuildDirectory),
            Some(15) => Ok(ChannelKind::GuildForum),
            _ => Err(Model(ModelError::InvalidPayload("Invalid channel kind".into()))),
        }
    }
}

impl ChannelKind {
    pub(crate) fn to_json(&self) -> Value {
        Value::Number(match self {
            ChannelKind::GuildText => 0.into(),
            ChannelKind::Dm => 1.into(),
            ChannelKind::GuildVoice => 2.into(),
            ChannelKind::GroupDm => 3.into(),
            ChannelKind::GuildCategory => 4.into(),
            ChannelKind::GuildAnnouncement => 5.into(),
            ChannelKind::AnnouncementThread => 10.into(),
            ChannelKind::PublicThread => 11.into(),
            ChannelKind::PrivateThread => 12.into(),
            ChannelKind::GuildStageVoice => 13.into(),
            ChannelKind::GuildDirectory => 14.into(),
            ChannelKind::GuildForum => 15.into(),
        })
    }
}

/// Represents a permission overwrite.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#overwrite-object-overwrite-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PermissionOverwrite {
    pub id: String,
    pub kind: PermissionOverwriteKind,
    pub allow: String,
    pub deny: String,
}

impl HttpRessource for PermissionOverwrite {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = raw["id"].as_str().ok_or(Model(ModelError::InvalidPayload("Invalid permission overwrite id".into())))?.to_string();
        let kind = PermissionOverwriteKind::from_raw(raw["type"].clone(), shard)?;
        let allow = raw["allow"].as_str().ok_or(Model(ModelError::InvalidPayload("Invalid permission overwrite allow".into())))?.to_string();
        let deny = raw["deny"].as_str().ok_or(Model(ModelError::InvalidPayload("Invalid permission overwrite deny".into())))?.to_string();

        Ok(PermissionOverwrite {
            id,
            kind,
            allow,
            deny,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum PermissionOverwriteKind {
    Role,
    Member,
}

impl HttpRessource for PermissionOverwriteKind {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(0) => Ok(PermissionOverwriteKind::Role),
            Some(1) => Ok(PermissionOverwriteKind::Member),
            _ => Err(Model(ModelError::InvalidPayload("Invalid permission overwrite kind".into()))),
        }
    }
}

/// Represents a thread metadata.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#thread-metadata-object-thread-metadata-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ThreadMetadata {
    pub archived: bool,
    pub auto_archive_duration: u64,
    pub archive_timestamp: chrono::DateTime<Utc>,
    pub locked: bool,
    pub invitable: Option<bool>,
    pub create_timestamp: chrono::DateTime<Utc>,
}

impl HttpRessource for ThreadMetadata {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        let archived = raw["archived"].as_bool().ok_or(Model(ModelError::InvalidPayload("Invalid thread metadata archived".into())))?;
        let auto_archive_duration = raw["auto_archive_duration"].as_u64().ok_or(Model(ModelError::InvalidPayload("Invalid thread metadata auto archive duration".into())))?;
        let timestamp = if let Some(ts) = raw["archive_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => t.with_timezone(&Utc),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            Utc::now()
        };

        let locked = raw["locked"].as_bool().ok_or(Model(ModelError::InvalidPayload("Invalid thread metadata locked".into())))?;
        let invitable = raw["invitable"].as_bool();
        let create_timestamp = if let Some(ts) = raw["archive_timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => t.with_timezone(&Utc),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            Utc::now()
        };

        Ok(Self {
            archived,
            auto_archive_duration,
            archive_timestamp: timestamp,
            locked,
            invitable,
            create_timestamp
        })
    }
}