use serde::{Deserialize, Serialize};
#[allow(unused)]
use crate::constants::API_VERSION;
use crate::models::channel::ChannelId;
use crate::models::guild::{Guild, GuildId, GuildMember};
use crate::models::interaction::Interaction;
use crate::models::message::Message;
use crate::models::Snowflake;

/// Contains data received by the websocket server when the client is ready
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Represents an event that is sent when a guild is created
///
/// Can be called:
/// - When the shard is initialized, to lazy load and back-fill information for all unavailable guilds sent in the `ready` event.
/// - When a guild becomes available again to the client.
/// - When the client joins a new guild.
///
/// Reference:
/// https://discord.com/developers/docs/topics/gateway-events#guild-create
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Represents an event that is sent when a guild is deleted
///
/// Can be called:
/// - When the client is removed from a guild
/// - When a guild becomes unavailable due to an outage, or when the client leaves or is removed from a guild
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildDelete {
    /// If this guild is unavailable
    ///
    /// The client is no longer in this guild when this field is false
    pub unavailable: bool,
    pub id: GuildId,
    /// The shard id
    pub shard: u64,
}

/// Represents an event that is sent when a message is created
///
/// Reference:
/// - [Message Create](https://discord.com/developers/docs/topics/gateway#message-create)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreate {
    pub message: Message,
    pub guild_id: Option<GuildId>,
    pub shard: u64,
}

/// Represents an event that is sent when a message is deleted
///
/// Reference:
/// - [Message Delete](https://discord.com/developers/docs/topics/gateway#message-delete)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelete {
    pub guild_id: Option<GuildId>,
    pub channel_id: ChannelId,
    pub id: Snowflake,
    pub shard: u64,
}

/// Represents an event that is sent when a member joins a guild
///
/// Reference:
/// - [Guild Member Add](https://discord.com/developers/docs/topics/gateway-events#guild-member-add)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberAdd {
    pub member: GuildMember,
    pub guild_id: GuildId,
    pub shard: u64,
}

/// Represents an event that is received when an interaction is created
///
/// Reference:
/// - [Interaction Create](https://discord.com/developers/docs/topics/gateway-events#interaction-create)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionCreate {
    pub interaction: Interaction,
    pub shard: u64,
}


/// Represents an event that is received when a guild member is updated
///
/// Reference:
/// - [Guild Member Add](https://discord.com/developers/docs/topics/gateway-events#guild-member-update)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildMemberUpdate {
    pub shard: u64,
    pub member: GuildMember
}