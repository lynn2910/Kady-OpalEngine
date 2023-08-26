use serde_json::Value;
use crate::constants::API_VERSION;
use error::{Error, EventError, Result};
use crate::manager::http::HttpRessource;
use crate::models::channel::ChannelId;
use crate::models::guild::{Guild, GuildId, GuildMember};
use crate::models::interaction::Interaction;
use crate::models::message::Message;
use crate::models::Snowflake;

/// Contains data received by the websocket server when the client is ready
pub struct Ready {
    /// Exact time at which the client was ready
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The shard id
    pub shard: u64,
    ///// The user that the client is logged in as
    //pub user: ClientUser,
    ///// The application that the client is logged in as
    //pub application: Application,
}

impl HttpRessource for Ready {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        if shard.is_none() {
            return Err(Error::Event(EventError::Runtime("Shard id is required to create a Ready event".to_string())));
        }

        if raw["v"].as_u64() != Some(API_VERSION) { panic!("API version mismatch"); }

        Ok(Self {
            timestamp: chrono::Utc::now(),
            //user: ClientUser::from_raw(raw["user"].take())?,
            //application: Application::from_raw(raw["application"].take())?,
            shard: shard.unwrap(),
        })
    }
}

/// Represents an event that is sent when a guild is created
///
/// Can be called:
/// - When the shard is initialized, to lazy load and backfill information for all unavailable guilds sent in the `ready` event.
/// - When a guild becomes available again to the client.
/// - When the client joins a new guild.
///
/// Reference:
/// https://discord.com/developers/docs/topics/gateway#guild-create
#[derive(Debug)]
pub struct GuildCreate {
    /// When this guild was joined at
    pub joined_at: chrono::DateTime<chrono::Utc>,
    /// Whether this guild is large
    pub large: bool,
    /// The approximate number of members in this guild
    pub member_count: u64,
    /// The id of the guild
    pub id: String,
    /// If this guild is unavailable
    pub unavailable: bool,
    /// If this guild is available, will be a `Guild` object, otherwise `None`
    pub guild: Option<Guild>,
    /// The shard id
    pub shard: u64,
}

impl HttpRessource for GuildCreate {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        if shard.is_none() {
            return Err(Error::Event(EventError::Runtime("Shard id is required to create a GuildCreate event".to_string())));
        }

        let large = raw["large"].as_bool().unwrap_or(false);
        let member_count = raw["member_count"].as_u64().unwrap_or(0);
        let id = raw["id"].as_str().unwrap_or("").to_string();
        let unavailable = raw["unavailable"].as_bool().unwrap_or(false);
        let guild = if unavailable { None } else { Some(Guild::from_raw(raw, shard)?) };

        Ok(Self {
            joined_at: chrono::Utc::now(),
            shard: shard.unwrap(),
            large,
            member_count,
            id,
            unavailable,
            guild
        })
    }
}

/// Represents an event that is sent when a message is created
///
/// Reference:
/// - [Message Create](https://discord.com/developers/docs/topics/gateway#message-create)
pub struct MessageCreate {
    pub message: Message,
    pub guild_id: Option<GuildId>,
    pub shard: u64,
}

impl HttpRessource for MessageCreate {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        if shard.is_none() {
            return Err(Error::Event(EventError::Runtime("Shard id is required to create a MessageCreate event".to_string())));
        }

        let guild_id = if let Some(guild_id) = raw.get("guild_id") { Some(GuildId::from_raw(guild_id.clone(), shard)?) } else { None };

        Ok(Self {
            message: Message::from_raw(raw, shard)?,
            shard: shard.unwrap(),
            guild_id
        })
    }
}

/// Represents an event that is sent when a message is deleted
///
/// Reference:
/// - [Message Delete](https://discord.com/developers/docs/topics/gateway#message-delete)
#[derive(Debug)]
pub struct MessageDelete {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub id: Snowflake,
    pub shard: u64,
}

impl HttpRessource for MessageDelete {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        if shard.is_none() {
            return Err(Error::Event(EventError::Runtime("Shard id is required to create a MessageDelete event".to_string())));
        }

        let guild_id = if let Some(guild_id) = raw.get("guild_id") { Some(GuildId::from_raw(guild_id.clone(), shard)?) } else { None };
        let channel_id = ChannelId::from_raw(raw["channel_id"].clone(), shard)?;
        let id = Snowflake::from_raw(raw["id"].clone(), shard)?;

        Ok(Self {
            channel_id,
            id,
            guild_id,
            shard: shard.unwrap()
        })
    }
}

/// Represents an event that is sent when a member joins a guild
///
/// Reference:
/// - [Guild Member Add](https://discord.com/developers/docs/topics/gateway-events#guild-member-add)
pub struct GuildMemberAdd {
    pub member: GuildMember,
    pub guild_id: GuildId,
    pub shard: u64,
}

impl HttpRessource for GuildMemberAdd {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        if shard.is_none() {
            return Err(Error::Event(EventError::Runtime("Shard id is required to create a GuildMemberAdd event".to_string())));
        }

        let guild_id = GuildId::from_raw(raw["guild_id"].clone(), shard)?;

        Ok(Self {
            member: GuildMember::from_raw(raw, shard)?,
            guild_id,
            shard: shard.unwrap()
        })
    }
}


/// Represents an event that is received when an interaction is created
///
/// Reference:
/// - [Interaction Create](https://discord.com/developers/docs/topics/gateway-events#interaction-create)
pub struct InteractionCreate {
    pub interaction: Interaction,
    pub shard: u64,
}

impl HttpRessource for InteractionCreate {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        if shard.is_none() {
            return Err(Error::Event(EventError::Runtime("Shard id is required to create an InteractionCreate event".to_string())));
        }

        Ok(Self {
            interaction: Interaction::from_raw(raw, shard)?,
            shard: shard.unwrap()
        })
    }
}



/// Represents an event that is received when a guild member is updated
///
/// Reference:
/// - [Guild Member Add](https://discord.com/developers/docs/topics/gateway-events#guild-member-update)
pub struct GuildMemberUpdate {
    pub shard: u64,
    pub member: GuildMember
}

impl HttpRessource for GuildMemberUpdate {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        if shard.is_none() {
            return Err(Error::Event(EventError::Runtime("Shard id is required to create an InteractionCreate event".to_string())));
        };

        Ok(Self {
            member: GuildMember::from_raw(raw, shard)?,
            shard: shard.unwrap(),
        })
    }
}