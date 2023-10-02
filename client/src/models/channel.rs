#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use chrono::Utc;
use serde::{Serialize, Deserialize, Deserializer};
use serde::de::Error;
use serde_json::Value;
use error::Result;
use crate::manager::cache::{CacheDock, UpdateCache};
use crate::manager::http::{ApiResult, Http};
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
        http.send_message(self, payload, None).await
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
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
    #[serde(rename = "type")]
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

/// Represents a guild text channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildTextChannel {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub name: Option<String>,
    pub guild_id: Option<GuildId>,
    pub position: Option<u64>,
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    pub topic: Option<String>,
    pub nsfw: Option<bool>,
    /// The id of the last message sent in this channel (may not point to an existing or valid message)
    pub last_message_id: Option<Snowflake>,
    /// Amount of seconds a user has to wait before sending another message (0-21600)
    pub rate_limit_per_user: Option<u64>,
    pub parent_id: Option<ChannelId>,
    
    pub last_pin_timestamp: Option<chrono::DateTime<Utc>>,
    /// Contain the permissions for the user in the channel, including overwrites, only when part of the interaction object
    pub permissions: Option<String>,
    pub flags: Option<u64>,


    #[serde(default = "crate::manager::cache::default_cache_dock")]
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

/// Represents a DM channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Dm {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub last_message_id: Option<String>,
    pub icon: Option<String>,
    
    pub last_pin_timestamp: Option<chrono::DateTime<Utc>>,
    /// Contain the permissions for the user in the channel, including overwrites, only when part of the interaction object
    pub permissions: Option<String>,

    #[serde(default = "crate::manager::cache::default_cache_dock")]
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

/// Represents a guild voice channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildVoice {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub name: Option<String>,
    pub guild_id: Option<GuildId>,
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

/// Represents a guild category channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildCategory {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub name: Option<String>,
    pub guild_id: Option<GuildId>,
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

/// Represents a guild announcement channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildAnnouncement {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub name: Option<String>,
    pub guild_id: Option<GuildId>,
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

    #[serde(default = "crate::manager::cache::default_cache_dock")]
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

/// Represents an announcement thread channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AnnouncementThread {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: Option<GuildId>,
    pub parent_id: Option<ChannelId>,
    pub creator_id: String,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub last_message_id: Option<String>,
    pub locked: bool,
    pub auto_archive_duration: u64,
    
    pub archive_timestamp: Option<chrono::DateTime<Utc>>,
    
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

/// Represents a public thread channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PublicThread {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: Option<GuildId>,
    pub parent_id: Option<ChannelId>,
    pub creator_id: String,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub last_message_id: Option<String>,
    pub locked: bool,
    pub auto_archive_duration: u64,
    
    pub archive_timestamp: Option<chrono::DateTime<Utc>>,
    
    pub locked_timestamp: Option<chrono::DateTime<Utc>>,
    pub message_count: u64,
    pub member_count: u64,
    pub thread_metadata: ThreadMetadata,
    pub default_auto_archive_duration: u64,

    #[serde(default = "crate::manager::cache::default_cache_dock")]
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

/// Represents a private thread channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PrivateThread {
    pub id: ChannelId,
    pub name: Option<String>,
    pub guild_id: Option<GuildId>,
    pub parent_id: Option<ChannelId>,
    pub creator_id: String,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub last_message_id: Option<String>,
    pub locked: bool,
    pub auto_archive_duration: u64,
    
    pub archive_timestamp: Option<chrono::DateTime<Utc>>,
    
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

/// Represents a guild stage voice channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildStageVoice {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub name: Option<String>,
    pub guild_id: Option<GuildId>,
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

    #[serde(default = "crate::manager::cache::default_cache_dock")]
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

/// Represents a guild forum channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildForum {
    pub id: ChannelId,
    #[serde(rename = "type")]
    pub kind: ChannelKind,
    pub name: Option<String>,
    pub guild_id: Option<GuildId>,
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

    #[serde(default = "crate::manager::cache::default_cache_dock")]
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

/// Represent every kind of channel.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#channel-object-channel-types)
#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
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

impl<'de> Deserialize<'de> for ChannelKind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        let value: u64 = Deserialize::deserialize(deserializer)?;

        match value {
            0 => Ok(Self::GuildText),
            1 => Ok(Self::Dm),
            2 => Ok(Self::GuildVoice),
            3 => Ok(Self::GroupDm),
            4 => Ok(Self::GuildCategory),
            5 => Ok(Self::GuildAnnouncement),
            10 => Ok(Self::AnnouncementThread),
            11 => Ok(Self::PublicThread),
            12 => Ok(Self::PrivateThread),
            13 => Ok(Self::GuildStageVoice),
            14 => Ok(Self::GuildDirectory),
            15 => Ok(Self::GuildForum),
            _ => Err(D::Error::custom(format!("unknown channel kind: {}", value)))
        }
    }
}

impl ChannelKind {
    pub(crate) fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

/// Represents a permission overwrite.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#overwrite-object-overwrite-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct PermissionOverwrite {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: PermissionOverwriteKind,
    pub allow: String,
    pub deny: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum PermissionOverwriteKind {
    Role = 0,
    Member = 1,
}

/// Represents a thread metadata.
///
/// Reference:
/// - [Discord Docs](https://discord.com/developers/docs/resources/channel#thread-metadata-object-thread-metadata-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ThreadMetadata {
    pub archived: bool,
    pub auto_archive_duration: u64,
    
    pub archive_timestamp: Option<chrono::DateTime<Utc>>,
    pub locked: bool,
    pub invitable: Option<bool>,
    
    pub create_timestamp: Option<chrono::DateTime<Utc>>,
}