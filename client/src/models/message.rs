#![deny(clippy::unwrap_in_result)]
#![deny(clippy::expect_used)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::unwrap_used)]

use std::str::FromStr;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use error::{ModelError, Result};
use error::Error::Model;
use crate::manager::cache::UpdateCache;
use crate::manager::http::HttpRessource;
use crate::models::channel::{ChannelId, ChannelKind, Thread};
use crate::models::components::message_components::Component;
use crate::models::components::embed::Embed;
use crate::models::components::Emoji;
use crate::models::components::sticker::StickerFormatType;
use crate::models::interaction::InteractionType;
use crate::models::Snowflake;
use crate::models::user::{User, UserId};

/// Represents a message in a channel
///
/// Reference:
/// - [Message Structure](https://discord.com/developers/docs/resources/channel#message-object-message-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Message {
    pub id: Snowflake,
    pub channel_id: ChannelId,
    pub author: User,
    pub content: Option<String>,
    /// When this message was sent
    pub timestamp: DateTime<Utc>,
    /// When this message was edited
    pub edited_timestamp: Option<DateTime<Utc>>,
    /// Whether this message mentions everyone
    pub mention_everyone: bool,
    /// Users specifically mentioned in the message
    pub mentions: Vec<User>,
    // TODO
    //pub mention_roles: Vec<Role>,

    /// Channels specifically mentioned in this message
    ///
    /// Not all channel mentions in a message will appear in mention_channels.
    /// Only textual channels that are visible to everyone in a lurkable guild will ever be included.
    pub mention_channels: Option<Vec<ChannelMention>>,

    /// Any attachments included in the message
    pub attachments: Vec<Attachment>,

    pub embeds: Vec<Embed>,
    pub sticker_items: Vec<StickerItem>,
    pub reactions: Vec<Reaction>,
    pub components: Vec<Component>,

    pub pinned: bool,
    pub webhook_id: Option<Snowflake>,
    /// The type of message
    pub kind: MessageType,

    pub application_id: Option<Snowflake>,

    pub message_reference: Option<MessageReference>,

    /// Contain message flags ORd together
    ///
    /// Reference:
    /// - [Message Flags](https://discord.com/developers/docs/resources/channel#message-object-message-flags)
    // TODO
    pub flags: Option<u64>,

    pub referenced_message: Option<Box<Message>>,
    pub interaction: Option<MessageInteraction>,
    pub thread: Option<Thread>,

    /// A generally increasing integer (there may be gaps or duplicates) that represents the approximate
    /// position of the message in a thread, it can be used to estimate the relative position of the
    /// message in a thread in company with total_message_sent on parent thread
    pub position: Option<u64>,
    pub role_subscription_data: Option<RoleSubscription>
}

impl UpdateCache for Message {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.channel_id != from.channel_id {
            self.channel_id = from.channel_id.clone();
        }

        self.author.update(&from.author);

        if self.content != from.content {
            self.content = from.content.clone();
        }
        if self.timestamp != from.timestamp {
            self.timestamp = from.timestamp;
        }
        if self.edited_timestamp != from.edited_timestamp {
            self.edited_timestamp = from.edited_timestamp;
        }

        if self.mention_everyone != from.mention_everyone {
            self.mention_everyone = from.mention_everyone;
        }

        if self.mentions != from.mentions {
            self.mentions = from.mentions.clone();
        }

        if self.mention_channels != from.mention_channels {
            self.mention_channels = from.mention_channels.clone();
        }

        if self.attachments != from.attachments {
            self.attachments = from.attachments.clone();
        }

        if self.embeds != from.embeds {
            self.embeds = from.embeds.clone();
        }

        if self.sticker_items != from.sticker_items {
            self.sticker_items = from.sticker_items.clone();
        }

        if self.reactions != from.reactions {
            self.reactions = from.reactions.clone();
        }

        if self.components != from.components {
            self.components = from.components.clone();
        }

        if self.pinned != from.pinned {
            self.pinned = from.pinned;
        }

        if self.webhook_id != from.webhook_id {
            self.webhook_id = from.webhook_id.clone();
        }

        if self.kind != from.kind {
            self.kind = from.kind.clone();
        }

        if self.application_id != from.application_id {
            self.application_id = from.application_id.clone();
        }

        if self.message_reference != from.message_reference {
            self.message_reference = from.message_reference.clone();
        }

        if self.flags != from.flags {
            self.flags = from.flags;
        }

        if self.referenced_message != from.referenced_message {
            self.referenced_message = from.referenced_message.clone();
        }

        if self.interaction != from.interaction {
            self.interaction = from.interaction.clone();
        }

        if self.thread != from.thread {
            self.thread = from.thread.clone();
        }

        if self.position != from.position {
            self.position = from.position;
        }

        if self.role_subscription_data != from.role_subscription_data {
            self.role_subscription_data = from.role_subscription_data.clone();
        }
    }
}

impl HttpRessource for Message {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = Snowflake::from_raw(raw["id"].clone(), shard)?;
        let channel_id = ChannelId::from_raw(raw["channel_id"].clone(), shard)?;
        let author = User::from_raw(raw["author"].clone(), shard)?;
        let content = raw["content"].as_str().map(|s| s.to_string());
        let timestamp = if let Some(ts) = raw["timestamp"].as_u64() {
            match Utc.timestamp_millis_opt(ts as i64) {
                chrono::LocalResult::Single(t) => t.with_timezone(&Utc),
                _ => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            Utc::now()
        };
        let edited_timestamp: Option<DateTime<Utc>> = if let Some(ts) = raw["edited_timestamp"].as_str() {
            match DateTime::from_str(ts) {
                Ok(t) => Some(t),
                Err(_) => return Err(Model(ModelError::InvalidTimestamp(format!("Invalid timestamp: {}", ts))))
            }
        } else {
            None
        };
        let mention_everyone = raw["mention_everyone"].as_bool().unwrap_or(false);
        let mentions = if let Some(raw_mentions) = raw["mentions"].as_array() {
            let mut mentions: Vec<User> = Vec::new();
            for mention in raw_mentions.iter() {
                if let Ok(mention) = User::from_raw(mention.clone(), shard) {
                    mentions.push(mention);
                }
            };
            mentions
        } else {
            Vec::new()
        };
        let mention_channels = if let Some(raw_mention_channels) = raw["mention_channels"].as_array() {
            let mut mention_channels: Vec<ChannelMention> = Vec::new();
            for mention_channel in raw_mention_channels.iter() {
                if let Ok(mention_channel) = ChannelMention::from_raw(mention_channel.clone(), shard) {
                    mention_channels.push(mention_channel);
                }
            };
            Some(mention_channels)
        } else {
            None
        };
        let attachments = if let Some(raw_attachments) = raw["attachments"].as_array() {
            let mut attachments: Vec<Attachment> = Vec::new();
            for attachment in raw_attachments.iter() {
                if let Ok(attachment) = Attachment::from_raw(attachment.clone(), shard) {
                    attachments.push(attachment);
                }
            };
            attachments
        } else {
            Vec::new()
        };
        let embeds = if let Some(raw_embeds) = raw["embeds"].as_array() {
            let mut embeds: Vec<Embed> = Vec::new();
            for embed in raw_embeds.iter() {
                if let Ok(embed) = Embed::from_raw(embed.clone(), shard) {
                    embeds.push(embed);
                }
            };
            embeds
        } else {
            Vec::new()
        };
        let components = if let Some(components) = raw.get("components") {
            if let Some(raw_components) = components.as_array() {
                let mut components = Vec::new();
                for component in raw_components.iter() {
                    if let Ok(component) = Component::from_raw(component.clone(), shard) {
                        components.push(component);
                    }
                };
                components
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let sticker_items = if let Some(sticker_items) = raw.get("sticker_items") {
            if let Some(sticker_items) = sticker_items.as_array() {
                let mut stickers = Vec::new();
                for sticker in sticker_items.iter() {
                    if let Ok(sticker) = StickerItem::from_raw(sticker.clone(), shard) {
                        stickers.push(sticker);
                    }
                };
                stickers
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let reactions = if let Some(reactions) = raw.get("reactions") {
            if let Some(raw_reactions) = reactions.as_array() {
                let mut reactions = Vec::new();
                for reaction in raw_reactions.iter() {
                    if let Ok(reaction) = Reaction::from_raw(reaction.clone(), shard) {
                        reactions.push(reaction);
                    }
                };
                reactions
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let pinned = raw["pinned"].as_bool().unwrap_or(false);
        let webhook_id = Snowflake::from_raw(raw["webhook_id"].clone(), shard).ok();
        let kind = if let Some(kind) = raw.get("type") { MessageType::from_raw(kind.clone(), shard)? } else { return Err(Model(ModelError::InvalidPayload("Message".to_string()))); };
        let application_id = Snowflake::from_raw(raw["application_id"].clone(), shard).ok();
        let message_reference = if let Some(message_reference) = raw.get("message_reference") {
            MessageReference::from_raw(message_reference.clone(), shard).ok()
        } else {
            None
        };
        let flags = raw["flags"].as_u64();
        let referenced_message = if let Some(referenced_message) = raw.get("referenced_message") {
            Message::from_raw(referenced_message.clone(), shard).ok().map(Box::new)
        } else {
            None
        };
        let interaction = if let Some(interaction) = raw.get("interaction") {
            MessageInteraction::from_raw(interaction.clone(), shard).ok()
        } else {
            None
        };
        // TODO
        let thread = None; // raw["thread"].as_object().map(|o| Thread::from_raw(o.clone()).unwrap());

        let position = raw["position"].as_u64();
        let role_subscription_data = if let Some(role_subscription_data) = raw.get("role_subscription_data") {
            RoleSubscription::from_raw(role_subscription_data.clone(), shard).ok()
        } else {
            None
        };

        Ok(Self {
            id,
            channel_id,
            author,
            content,
            timestamp,
            edited_timestamp,
            mention_everyone,
            mentions,
            mention_channels,
            attachments,
            embeds,
            sticker_items,
            reactions,
            components,
            pinned,
            webhook_id,
            kind,
            application_id,
            message_reference,
            flags,
            referenced_message,
            interaction,
            thread,
            position,
            role_subscription_data
        })
    }
}

/// Represents the type of a message
///
/// Reference:
/// - [Message Types](https://discord.com/developers/docs/resources/channel#message-object-message-types)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum MessageType {
    Default = 0,
    RecipientAdd = 1,
    RecipientRemove = 2,
    Call = 3,
    ChannelNameChange = 4,
    ChannelIconChange = 5,
    ChannelPinnedMessage = 6,
    UserJoin = 7,
    GuildBoost = 8,
    GuildBoostTier1 = 9,
    GuildBoostTier2 = 10,
    GuildBoostTier3 = 11,
    ChannelFollowAdd = 12,
    GuildDiscoveryDisqualified = 14,
    GuildDiscoveryReQualified = 15,
    GuildDiscoveryGracePeriodInitialWarning = 16,
    GuildDiscoveryGracePeriodFinalWarning = 17,
    ThreadCreated = 18,
    Reply = 19,
    ChatInputCommand = 20,
    ThreadStarterMessage = 21,
    GuildInviteReminder = 22,
    ContextMenuCommand = 23,
    AutoModerationAction = 24,
    RoleSubscriptionPurchase = 25,
    InteractionPremiumUpSell = 26,
    StageStart = 27,
    StageEnd = 28,
    StageSpeaker = 29,
    StageTopic = 31,
    GuildApplicationPremiumSubscription = 32,
}

impl HttpRessource for MessageType {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        match raw.as_u64() {
            Some(0) => Ok(Self::Default),
            Some(1) => Ok(Self::RecipientAdd),
            Some(2) => Ok(Self::RecipientRemove),
            Some(3) => Ok(Self::Call),
            Some(4) => Ok(Self::ChannelNameChange),
            Some(5) => Ok(Self::ChannelIconChange),
            Some(6) => Ok(Self::ChannelPinnedMessage),
            Some(7) => Ok(Self::UserJoin),
            Some(8) => Ok(Self::GuildBoost),
            Some(9) => Ok(Self::GuildBoostTier1),
            Some(10) => Ok(Self::GuildBoostTier2),
            Some(11) => Ok(Self::GuildBoostTier3),
            Some(12) => Ok(Self::ChannelFollowAdd),
            Some(14) => Ok(Self::GuildDiscoveryDisqualified),
            Some(15) => Ok(Self::GuildDiscoveryReQualified),
            Some(16) => Ok(Self::GuildDiscoveryGracePeriodInitialWarning),
            Some(17) => Ok(Self::GuildDiscoveryGracePeriodFinalWarning),
            Some(18) => Ok(Self::ThreadCreated),
            Some(19) => Ok(Self::Reply),
            Some(20) => Ok(Self::ChatInputCommand),
            Some(21) => Ok(Self::ThreadStarterMessage),
            Some(22) => Ok(Self::GuildInviteReminder),
            Some(23) => Ok(Self::ContextMenuCommand),
            Some(24) => Ok(Self::AutoModerationAction),
            Some(25) => Ok(Self::RoleSubscriptionPurchase),
            Some(26) => Ok(Self::InteractionPremiumUpSell),
            Some(27) => Ok(Self::StageStart),
            Some(28) => Ok(Self::StageEnd),
            Some(29) => Ok(Self::StageSpeaker),
            Some(31) => Ok(Self::StageTopic),
            Some(32) => Ok(Self::GuildApplicationPremiumSubscription),
            _ => Err(Model(ModelError::InvalidPayload("Invalid message type".to_owned()))),
        }
    }
}

/// Represents a channel mention in a message
///
/// Reference:
/// - [Channel Mention Structure](https://discord.com/developers/docs/resources/channel#channel-mention-object-channel-mention-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ChannelMention {
    pub id: Snowflake,
    pub guild_id: Snowflake,
    pub channel_type: ChannelKind,
    pub name: String,
}

impl HttpRessource for ChannelMention {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = Snowflake::from_raw(raw["id"].clone(), shard)?;
        let guild_id = Snowflake::from_raw(raw["guild_id"].clone(), shard)?;
        let channel_type = ChannelKind::from_raw(raw["type"].clone(), shard)?;
        let name = if let Some(name) = raw["name"].as_str() { name } else { return Err(Model(ModelError::MissingField("Field 'name' is missing for the channel mention".into()))); }.to_owned();

        Ok(Self {
            id,
            guild_id,
            channel_type,
            name,
        })
    }
}

/// Represents an attachment in a message
///
/// Reference:
/// - [Attachment Structure](https://discord.com/developers/docs/resources/channel#attachment-object-attachment-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Attachment {
    pub id: Snowflake,
    pub filename: String,
    pub description: Option<String>,
    /// The content type of the file
    ///
    /// Based on the mime type
    pub content_type: String,
    pub size: u64,
    pub url: String,
    pub proxy_url: String,
    pub height: Option<u64>,
    pub width: Option<u64>,
    pub ephemeral: Option<bool>,
    /// The duration of the audio file (currently for voice messages)
    pub durations_seconds: Option<u64>,
}

impl UpdateCache for Attachment {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.filename != from.filename {
            self.filename = from.filename.clone();
        }
        if self.description != from.description {
            self.description = from.description.clone();
        }
        if self.content_type != from.content_type {
            self.content_type = from.content_type.clone();
        }
        if self.size != from.size {
            self.size = from.size;
        }
        if self.url != from.url {
            self.url = from.url.clone();
        }
        if self.proxy_url != from.proxy_url {
            self.proxy_url = from.proxy_url.clone();
        }
        if self.height != from.height {
            self.height = from.height;
        }
        if self.width != from.width {
            self.width = from.width;
        }
        if self.ephemeral != from.ephemeral {
            self.ephemeral = from.ephemeral;
        }
        if self.durations_seconds != from.durations_seconds {
            self.durations_seconds = from.durations_seconds;
        }
    }
}

impl HttpRessource for Attachment {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = Snowflake::from_raw(raw["id"].clone(), shard)?;
        let filename = if let Some(filename) = raw["filename"].as_str() { filename } else { return Err(Model(ModelError::MissingField("Field 'filename' is missing for the attachment".into()))); };
        let description = raw["description"].as_str().map(|s| s.to_owned());
        let content_type = if let Some(content_type) = raw["content_type"].as_str() { content_type } else { return Err(Model(ModelError::MissingField("Field 'content_type' is missing for the attachment".into()))); };
        let size = if let Some(size) = raw["size"].as_u64() { size } else { return Err(Model(ModelError::MissingField("Field 'size' is missing for the attachment".into()))); };
        let url = if let Some(url) = raw["url"].as_str() { url } else { return Err(Model(ModelError::MissingField("Field 'url' is missing for the attachment".into()))); };
        let proxy_url = if let Some(proxy_url) = raw["proxy_url"].as_str() { proxy_url } else { return Err(Model(ModelError::MissingField("Field 'proxy_url' is missing for the attachment".into()))); };
        let height = raw["height"].as_u64();
        let width = raw["width"].as_u64();
        let ephemeral = raw["ephemeral"].as_bool();
        let durations_seconds = raw["durations_seconds"].as_u64();

        Ok(Self {
            id,
            filename: filename.to_owned(),
            description,
            content_type: content_type.to_owned(),
            size,
            url: url.to_owned(),
            proxy_url: proxy_url.to_owned(),
            height,
            width,
            ephemeral,
            durations_seconds,
        })
    }
}

/// Represents a reference to a message
///
/// Reference:
/// - [Message Reference Structure](https://discord.com/developers/docs/resources/channel#message-object-message-reference-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MessageReference {
    pub message_id: Option<Snowflake>,
    pub channel_id: Snowflake,
    pub guild_id: Option<Snowflake>,
    pub fail_if_not_exists: Option<bool>,
}

impl UpdateCache for MessageReference {
    fn update(&mut self, from: &Self) {
        if self.message_id != from.message_id {
            self.message_id = from.message_id.clone();
        }
        if self.channel_id != from.channel_id {
            self.channel_id = from.channel_id.clone();
        }
        if self.guild_id != from.guild_id {
            self.guild_id = from.guild_id.clone();
        }
        if self.fail_if_not_exists != from.fail_if_not_exists {
            self.fail_if_not_exists = from.fail_if_not_exists;
        }
    }
}

impl HttpRessource for MessageReference {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let message_id = if let Some(message_id) = raw.get("message_id") { Some(Snowflake::from_raw(message_id.clone(), shard)?) } else { None };
        let channel_id = Snowflake::from_raw(raw["channel_id"].clone(), shard)?;
        let guild_id = if let Some(guild_id) = raw.get("guild_id") { Some(Snowflake::from_raw(guild_id.clone(), shard)?) } else { None };
        let fail_if_not_exists = raw["fail_if_not_exists"].as_bool();

        Ok(Self {
            message_id,
            channel_id,
            guild_id,
            fail_if_not_exists,
        })
    }
}

/// If the message is a response to an Interaction, this is the id of the interaction's application
///
/// Reference:
/// - [Message Interaction Structure](https://discord.com/developers/docs/resources/channel#message-object-message-interaction-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MessageInteraction {
    pub id: Snowflake,
    pub kind: InteractionType,
    pub name: String,
    pub user: User,
}

impl UpdateCache for MessageInteraction {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.kind != from.kind {
            self.kind = from.kind.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.user != from.user {
            self.user = from.user.clone();
        }
    }
}

impl HttpRessource for MessageInteraction {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = Snowflake::from_raw(raw["id"].clone(), shard)?;
        let kind = if let Some(kind) = raw.get("type") { InteractionType::from_raw(kind.clone(), shard)? } else { return Err(Model(ModelError::MissingField("Field 'type' is missing for the message interaction".into()))); };
        let name = if let Some(name) = raw["name"].as_str() { name } else { return Err(Model(ModelError::MissingField("Field 'name' is missing for the message interaction".into()))); };
        let user = if let Some(user) = raw.get("user") { User::from_raw(user.clone(), shard)? } else { return Err(Model(ModelError::MissingField("Field 'user' is missing for the message interaction".into()))); };

        Ok(Self {
            id,
            kind,
            name: name.to_owned(),
            user,
        })
    }
}

/// Represents a role subscription purchase
///
/// Reference:
/// - [Role Subscription Structure](https://discord.com/developers/docs/resources/channel#role-subscription-data-object)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RoleSubscription {
    pub tier_name: String,
    pub role_subscription_listing_id: Snowflake,
    pub total_months_subscribed: u64,
    pub is_renewal: bool,
}

impl UpdateCache for RoleSubscription {
    fn update(&mut self, from: &Self) {
        if self.tier_name != from.tier_name {
            self.tier_name = from.tier_name.clone();
        }
        if self.total_months_subscribed != from.total_months_subscribed {
            self.total_months_subscribed = from.total_months_subscribed;
        }
        if self.is_renewal != from.is_renewal {
            self.is_renewal = from.is_renewal;
        }
        if self.role_subscription_listing_id != from.role_subscription_listing_id {
            self.role_subscription_listing_id = from.role_subscription_listing_id.clone();
        }
    }
}

impl HttpRessource for RoleSubscription {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let tier_name = if let Some(tier_name) = raw["tier_name"].as_str() { tier_name } else { return Err(Model(ModelError::MissingField("Field 'tier_name' is missing for the role subscription".into()))); };
        let role_subscription_listing_id = Snowflake::from_raw(raw["role_subscription_listing_id"].clone(), shard)?;
        let total_months_subscribed = if let Some(total_months_subscribed) = raw["total_months_subscribed"].as_u64() { total_months_subscribed } else { return Err(Model(ModelError::MissingField("Field 'total_months_subscribed' is missing for the role subscription".into()))); };
        let is_renewal = if let Some(is_renewal) = raw["is_renewal"].as_bool() { is_renewal } else { return Err(Model(ModelError::MissingField("Field 'is_renewal' is missing for the role subscription".into()))); };

        Ok(Self {
            tier_name: tier_name.to_owned(),
            role_subscription_listing_id,
            total_months_subscribed,
            is_renewal,
        })
    }
}

/// Represent a sticker item in a message
///
/// Reference:
/// - [Sticker Item Structure](https://discord.com/developers/docs/resources/channel#message-object-message-sticker-item-structure)
/// - [Sticker Format Type](https://discord.com/developers/docs/resources/channel#message-object-message-sticker-format-types)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct StickerItem {
    pub id: Snowflake,
    pub name: String,
    pub format_type: StickerFormatType,
}

impl UpdateCache for StickerItem {
    fn update(&mut self, from: &Self) {
        if self.id != from.id {
            self.id = from.id.clone();
        }
        if self.name != from.name {
            self.name = from.name.clone();
        }
        if self.format_type != from.format_type {
            self.format_type = from.format_type.clone();
        }
    }
}

impl HttpRessource for StickerItem {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = Snowflake::from_raw(raw["id"].clone(), shard)?;
        let name = if let Some(name) = raw["name"].as_str() { name } else { return Err(Model(ModelError::MissingField("Field 'name' is missing for the sticker item".into()))); };
        let format_type = if let Some(format_type) = raw.get("format_type") { StickerFormatType::from_raw(format_type.clone(), shard)? } else { return Err(Model(ModelError::MissingField("Field 'format_type' is missing for the sticker item".into()))); };

        Ok(Self {
            id,
            name: name.to_owned(),
            format_type,
        })
    }
}

/// Represents a reaction to a message
///
/// Reference:
/// - [Reaction Structure](https://discord.com/developers/docs/resources/channel#reaction-object-reaction-structure)
/// - [Emoji Structure](https://discord.com/developers/docs/resources/emoji#emoji-object-emoji-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Reaction {
    pub count: u64,
    /// Whether the current user reacted using this emoji
    pub me: bool,
    pub emoji: Emoji,
}

impl UpdateCache for Reaction {
    fn update(&mut self, from: &Self) {
        if self.count != from.count {
            self.count = from.count;
        }

        if self.me != from.me {
            self.me = from.me;
        }

        if self.emoji != from.emoji {
            self.emoji = from.emoji.clone();
        }
    }
}

impl HttpRessource for Reaction {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let count = if let Some(count) = raw["count"].as_u64() { count } else { return Err(Model(ModelError::MissingField("Field 'count' is missing for the reaction".into()))); };
        let me = if let Some(me) = raw["me"].as_bool() { me } else { return Err(Model(ModelError::MissingField("Field 'me' is missing for the reaction".into()))); };
        let emoji = Emoji::from_raw(raw["emoji"].clone(), shard)?;

        Ok(Self {
            count,
            me,
            emoji,
        })
    }
}



#[derive(Debug)]
pub struct MessageBuilder {
    pub content: Option<String>,
    pub embeds: Vec<Embed>,
    pub components: Vec<Component>,
    pub allowed_mentions: Option<AllowedMentions>,
    pub attachments: Vec<MessageAttachmentBuilder>,
    pub reference: Option<MessageReference>,
    pub ephemeral: bool,
    pub flags: MessageFlags
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self {
            content: None,
            embeds: Vec::new(),
            components: Vec::new(),
            allowed_mentions: None,
            attachments: Vec::new(),
            reference: None,
            ephemeral: false,
            flags: MessageFlags::new()
        }
    }
}

impl MessageBuilder {
    pub fn new() -> Self {
        MessageBuilder::default()
    }

    pub fn set_content(mut self, content: impl ToString) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn set_flags(mut self, flags: u64) -> Self {
        self.flags.0 = flags;
        self
    }

    pub fn set_ephemeral(mut self, ephemeral: bool) -> Self {
        if ephemeral {
            self.flags.add_flag(message_flags::EPHEMERAL);
        } else {
            self.flags.remove_flag(message_flags::EPHEMERAL)
        };
        self
    }

    pub fn set_loading(mut self, loading: bool) -> Self {
        if loading {
            self.flags.add_flag(message_flags::LOADING);
        } else {
            self.flags.remove_flag(message_flags::LOADING)
        };
        self
    }

    pub fn remove_content(&mut self) {
        self.content = None;
    }

    pub fn add_embed(mut self, embed: Embed) -> Self {
        self.embeds.push(embed);
        self
    }

    pub fn add_component(mut self, component: Component) -> Self {
        self.components.push(component);
        self
    }

    pub fn add_attachment(mut self, attachment: MessageAttachmentBuilder) -> Self {
        self.attachments.push(attachment);
        self
    }

    pub fn set_allowed_mentions(mut self, allowed_mentions: AllowedMentions) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    pub fn remove_allowed_mentions(mut self) -> Self {
        self.allowed_mentions = None;
        self
    }

    pub fn set_reference(mut self, reference: MessageReference) -> Self {
        self.reference = Some(reference);
        self
    }

    pub fn to_json(&self) -> Value {
        let mut json = json!({});

        if let Some(content) = &self.content {
            json["content"] = json!(content);
        }

        if !self.embeds.is_empty() {
            json["embeds"] = json!(self.embeds);
        }

        if !self.components.is_empty() {
            let mut array = Vec::new();

            for component in &self.components {
                array.push(component.to_json())
            };

            json["components"] = Value::Array(array);
        }

        if let Some(allowed_mentions) = &self.allowed_mentions {
            json["allowed_mentions"] = json!(allowed_mentions);
        }

        if !self.attachments.is_empty() {
            let mut vec = Vec::new();

            for attachment in &self.attachments {
                vec.push(attachment.to_json());
            }

            json["attachments"] = vec.into();
        }

        if let Some(reference) = &self.reference {
            json["message_reference"] = json!(reference);
        }

        if !self.flags.is_empty() {
            json["flags"] = self.flags.to_json();
        }

        json
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageAttachmentBuilder {
    pub description: Option<String>,
    pub name: String,
    pub content_type: String,
    pub id: u8
}

impl MessageAttachmentBuilder {
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "description": self.description,
            "name": self.name,
            "content_type": self.content_type,
            "id": self.id
        })
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttachmentBuilder {
    pub bytes: Vec<u8>,
    /// The mime-type
    pub content_type: String,

    pub description: Option<String>,
    pub filename: String,

    pub id: u8
}


/// Represent the flags of a message
#[derive(Debug, Default)]
pub struct MessageFlags(pub u64);

/// Contain every message flags possible
///
/// Reference:
/// - [Message Flags](https://discord.com/developers/docs/resources/channel#message-object-message-flags)
#[allow(dead_code)]
pub mod message_flags {
    pub const CROSSPOST: u64 = 1 << 0;
    pub const IS_CROSSPOST: u64 = 1 << 1;
    pub const SUPRESS_EMBEDS: u64 = 1 << 2;
    pub const SOURCE_MESSAGE_DELETED: u64 = 1 << 3;
    pub const URGENT: u64 = 1 << 4;
    pub const HAS_THREAD: u64 = 1 << 5;
    pub const EPHEMERAL: u64 = 1 << 6;
    pub const LOADING: u64 = 1 << 7;
    pub const FAILED_TO_MENTION_SOME_ROLES_IN_THREAD: u64 = 1 << 8;
    pub const SUPPRESS_NOTIFICATIONS: u64 = 1 << 12;
    pub const IS_VOICE: u64 = 1 << 13;
}

impl MessageFlags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Add a flag
    pub fn add_flag(&mut self, flag: u64) {
        self.0 |= flag;
    }

    /// Add multiple flags
    pub fn add_flags(&mut self, flags: &[u64]) {
        for &flag in flags {
            self.add_flag(flag);
        }
    }

    /// Remove a flag
    pub fn remove_flag(&mut self, flag: u64) {
        self.0 &= !flag;
    }

    /// Remove multiple flags
    pub fn remove_flags(&mut self, flags: &[u64]) {
        for &flag in flags {
            self.remove_flag(flag);
        }
    }

    pub fn to_json(&self) -> Value {
        self.0.into()
    }
}

/// Represents a reference to a message
///
/// Reference:
/// - [Message Reference Structure](https://discord.com/developers/docs/resources/channel#message-object-message-reference-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllowedMentions {
    pub roles: Vec<Snowflake>,
    pub users: Vec<UserId>,
    pub everyone: bool,
}

impl AllowedMentions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_role(&mut self, role: Snowflake) {
        self.roles.push(role);
    }

    pub fn add_user(&mut self, user: UserId) {
        self.users.push(user);
    }

    pub fn allow_everyone(&mut self) {
        self.everyone = true;
    }

    pub fn disallow_everyone(&mut self) {
        self.everyone = false;
    }
}