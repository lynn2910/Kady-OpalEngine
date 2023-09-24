use std::sync::Arc;
use tokio::sync::RwLock;
use crate::manager::cache::CacheManager;
use crate::manager::http::Http;
use crate::manager::shard::ShardManager;
use crate::models::events::{GuildCreate, GuildDelete, GuildMemberAdd, GuildMemberUpdate, InteractionCreate, MessageCreate, MessageDelete, Ready};
use crate::typemap::{Type, TypeMap};

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Called when a shard is ready
    ///
    /// Reference:
    /// - [Discord Docs - Gateway Events](https://discord.com/developers/docs/topics/gateway-events#ready)
    async fn ready(&self, _ctx: Context, _ready: Ready) {}

    /// Called when a guild is created
    ///
    /// Will be triggered:
    /// - When the shard is initialized, to lazy load and backfill information for all unavailable guilds sent in the `ready` event.
    /// - When a guild becomes available again to the client.
    /// - When the client joins a new guild.
    ///
    /// Reference:
    /// - [Discord Docs - Gateway Events](https://discord.com/developers/docs/topics/gateway-events#guild-create)
    async fn guild_create(&self, _ctx: Context, _payload: GuildCreate) {}

    /// Called when a guild is deleted or unavailable
    ///
    /// Will be triggered:
    /// - When the client is removed from a guild
    /// - When a guild become unavailable
    ///
    /// Reference:
    /// - [Discord Docs - Gateway Events](https://discord.com/developers/docs/topics/gateway-events#guild-create)
    async fn guild_delete(&self, _ctx: Context, _payload: GuildDelete) {}

    /// Called when a message is created
    ///
    /// Reference:
    /// - [Discord Docs - Gateway Events](https://discord.com/developers/docs/topics/gateway-events#message-create)
    async fn message_create(&self, _ctx: Context, _payload: MessageCreate) {}

    /// Called when a message is deleted
    ///
    /// Reference:
    /// - [Discord Docs - Gateway Events](https://discord.com/developers/docs/topics/gateway-events#message-delete)
    async fn message_delete(&self, _ctx: Context, _payload: MessageDelete) {}

    /// Called when a member joins a guild
    ///
    /// Reference:
    /// - [Discord Docs - Gateway Events](https://discord.com/developers/docs/topics/gateway-events#guild-member-add)
    async fn guild_member_add(&self, _ctx: Context, _payload: GuildMemberAdd) {}

    /// Called when a member is updated inside a guild
    ///
    /// Reference:
    /// - [Discord Docs - Gateway Events](https://discord.com/developers/docs/topics/gateway-events#guild-member-update)
    async fn guild_member_update(&self, _ctx: Context, _payload: GuildMemberUpdate) {}

    /// Called when the client initially starts
    async fn start(&self, _ctx: Context) {}

    /// Called when an interaction is created
    ///
    /// Reference:
    /// - [Discord Docs - Gateway Events](https://discord.com/developers/docs/topics/gateway-events#interaction-create)
    async fn interaction_create(&self, _ctx: Context, _payload: InteractionCreate) {}
}



#[derive(Clone)]
pub struct Context {
    pub data: Arc<RwLock<TypeMap>>,
    pub shard_id: u64,
    pub skynet: Arc<Http>,
    pub shard_manager: Arc<RwLock<ShardManager>>,
    pub cache: Arc<RwLock<CacheManager>>
}

impl Context {
    /// Clone the current context
    pub fn clone_context(&self) -> Self {
        Self {
            data: self.data.clone(),
            shard_id: self.shard_id,
            skynet: self.skynet.clone(),
            shard_manager: self.shard_manager.clone(),
            cache: self.cache.clone()
        }
    }

    /// Create a new context based on the current context
    pub fn new(
        data: Arc<RwLock<TypeMap>>,
        shard_id: u64,
        rest: Arc<Http>,
        shard_manager: Arc<RwLock<ShardManager>>,
        cache: Arc<RwLock<CacheManager>>
    ) -> Self {
        Self {
            data,
            shard_id,
            skynet: rest,
            shard_manager,
            cache
        }
    }

    /// Get the current user
    pub async fn get_client_user(&self) -> Option<crate::models::user::ClientUser> {
        let cache = self.cache.read().await;
        cache.get_client_user().cloned()
    }

    /// Insert a value inside the client, to use INSIDE the events
    pub async fn insert_data<T: Type>(&self, value: T) -> Option<Box<T>> {
        let mut docker = self.data.write().await;
        docker.insert::<T>(value)
    }

    /// Remove a element from the data container
    pub async fn remove_data<T: Type>(&self) -> Option<Box<T>> {
        let mut docker = self.data.write().await;
        docker.remove::<T>()
    }

    /// Access a element from the data, and return a cloned, if present
    pub async fn get_data<T: Type>(&self) -> Option<T> {
        let docker = self.data.read().await;
        docker.get::<T>().cloned()
    }
}