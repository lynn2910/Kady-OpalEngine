use std::collections::HashMap;
use std::hash::Hash;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use crate::models::channel::{Channel, ChannelId, Thread};
use crate::models::guild::{Guild, GuildId, GuildMember, Role};
use crate::models::message::Message;
use crate::models::Snowflake;
use crate::models::user::{Application, ClientUser, User, UserId};


/// This trait is used to update ressources in the cache
pub trait UpdateCache: Send + Sync + Clone + Eq + PartialEq {
    fn update(&mut self, from: &Self);
}

#[derive(Default)]
pub struct CacheManager {
    client_user: Option<ClientUser>,
    application: Option<Application>,
    guilds: HashMap<GuildId, Guild>,
    channels: HashMap<ChannelId, Channel>,
    users: HashMap<UserId, User>
}

impl CacheManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get_client_user_mem_size(&self) -> usize {
        std::mem::size_of_val(&self.client_user)
    }
    pub fn get_application_mem_size(&self) -> usize {
        std::mem::size_of_val(&self.application)
    }
    pub fn get_guild_mem_size(&self) -> usize {
        std::mem::size_of_val(&self.guilds)
    }
    pub fn get_channels_mem_size(&self) -> usize {
        std::mem::size_of_val(&self.channels)
    }
    pub fn get_users_mem_size(&self) -> usize {
        std::mem::size_of_val(&self.users)
    }

    /// Updates the client user in the cache.
    pub fn update_client_user(&mut self, client_user: &ClientUser) {
        if let Some(cache_client_user) = &mut self.client_user {
            cache_client_user.update(client_user)
        } else {
            self.client_user = Some(client_user.clone());
        }
    }

    /// Returns a reference to the client user if it exists.
    pub fn get_client_user(&self) -> Option<&ClientUser> {
        self.client_user.as_ref()
    }

    /// Updates the application id in the cache.
    pub fn update_application(&mut self, application: &Application) {
        if let Some(cache_application) = &mut self.application {
            cache_application.update(application)
        } else {
            self.application = Some(application.clone());
        }
    }

    /// Returns a reference to the application if it exists.
    pub fn get_application(&self) -> Option<&Application> {
        self.application.as_ref()
    }

    /// Add or update a guild in the cache.
    pub fn update_guild(&mut self, guild: &Guild) {
        if let Some(cache_guild) = self.guilds.get_mut(&guild.id) {
            cache_guild.update(guild)
        } else {
            self.guilds.insert(guild.id.clone(), guild.clone());
        }
    }

    /// Returns a reference to a guild if it exists.
    pub fn get_guild(&self, guild_id: &GuildId) -> Option<&Guild> {
        self.guilds.get(guild_id)
    }

    /// Add or update a channel in the cache.
    pub fn update_channel(&mut self, channel: &Channel) {
        let channel_id = match &channel {
            Channel::GuildText(channel) => channel.id.clone(),
            Channel::Dm(channel) => channel.id.clone(),
            Channel::GuildAnnoucement(channel) => channel.id.clone(),
            Channel::GuildForum(channel) => channel.id.clone(),
            Channel::GuildVoice(channel) => channel.id.clone(),
            Channel::Thread(thread) => {
                match thread {
                    Thread::PublicThread(thread) => thread.id.clone(),
                    Thread::PrivateThread(thread) => thread.id.clone(),
                    Thread::AnnouncementThread(thread) => thread.id.clone(),
                }
            },
            _ => return
        };

        if let Some(cache_channel) = self.channels.get_mut(&channel_id) {
            cache_channel.update(channel)
        } else {
            self.channels.insert(channel_id, channel.clone());
        }
    }

    /// Returns a reference to a channel if it exists.
    pub fn get_channel(&self, channel_id: &ChannelId) -> Option<&Channel> {
        self.channels.get(channel_id)
    }

    /// Add or update a message in the cache.
    pub fn update_message(&mut self, channel_id: &ChannelId, message: Message) {
        match self.channels.get_mut(channel_id) {
            Some(Channel::GuildText(channel)) => {
                if let Some(cache_message) = channel.messages.get_mut(&message.id) {
                    cache_message.update(&message)
                } else {
                    channel.messages.insert(message.id.clone(), message.clone());
                }
            },
            Some(Channel::Dm(channel)) => {
                if let Some(cache_message) = channel.messages.get_mut(&message.id) {
                    cache_message.update(&message)
                } else {
                    channel.messages.insert(message.id.clone(), message.clone());
                }
            },
            Some(Channel::GuildAnnoucement(channel)) => {
                if let Some(cache_message) = channel.messages.get_mut(&message.id) {
                    cache_message.update(&message)
                } else {
                    channel.messages.insert(message.id.clone(), message.clone());
                }
            },
            Some(Channel::GuildForum(channel)) => {
                if let Some(cache_message) = channel.messages.get_mut(&message.id) {
                    cache_message.update(&message)
                } else {
                    channel.messages.insert(message.id.clone(), message.clone());
                }
            },
            Some(Channel::Thread(thread)) => {
                match thread {
                    Thread::PublicThread(thread) => {
                        if let Some(cache_message) = thread.messages.get_mut(&message.id) {
                            cache_message.update(&message)
                        } else {
                            thread.messages.insert(message.id.clone(), message.clone());
                        }
                    },
                    Thread::PrivateThread(thread) => {
                        if let Some(cache_message) = thread.messages.get_mut(&message.id) {
                            cache_message.update(&message)
                        } else {
                            thread.messages.insert(message.id.clone(), message.clone());
                        }
                    },
                    Thread::AnnouncementThread(thread) => {
                        if let Some(cache_message) = thread.messages.get_mut(&message.id) {
                            cache_message.update(&message)
                        } else {
                            thread.messages.insert(message.id.clone(), message.clone());
                        }
                    },
                };
            },
            _ => {}
        }
    }

    /// Returns a reference to a message if it exists.
    pub fn get_message(&self, channel_id: &ChannelId, message_id: &Snowflake) -> Option<&Message> {
        match self.channels.get(channel_id) {
            Some(Channel::GuildText(channel)) => channel.messages.get(message_id),
            Some(Channel::Dm(channel)) => channel.messages.get(message_id),
            Some(Channel::GuildAnnoucement(channel)) => channel.messages.get(message_id),
            Some(Channel::GuildForum(channel)) => channel.messages.get(message_id),
            Some(Channel::Thread(thread)) => {
                match thread {
                    Thread::PublicThread(thread) => thread.messages.get(message_id),
                    Thread::PrivateThread(thread) => thread.messages.get(message_id),
                    Thread::AnnouncementThread(thread) => thread.messages.get(message_id),
                }
            },
            _ => None
        }
    }

    pub fn delete_message(&mut self, channel_id: &ChannelId, message_id: &Snowflake) {
        match self.channels.get_mut(channel_id) {
            Some(Channel::GuildText(channel)) => {
                channel.messages.remove(message_id);
            },
            Some(Channel::Dm(channel)) => {
                channel.messages.remove(message_id);
            },
            Some(Channel::GuildAnnoucement(channel)) => {
                channel.messages.remove(message_id);
            },
            Some(Channel::GuildForum(channel)) => {
                channel.messages.remove(message_id);
            },
            Some(Channel::Thread(thread)) => {
                match thread {
                    Thread::PublicThread(thread) => thread.messages.remove(message_id),
                    Thread::PrivateThread(thread) => thread.messages.remove(message_id),
                    Thread::AnnouncementThread(thread) => thread.messages.remove(message_id),
                };
            },
            _ => {}
        }
    }

    /// Add or update a user in the cache.
    pub fn update_user(&mut self, user: &User) {
        if let Some(cache_user) = self.users.get_mut(&user.id) {
            cache_user.update(user)
        } else {
            self.users.insert(user.id.clone(), user.clone());
        }
    }

    /// Returns a reference to a user if it exists.
    pub fn get_user(&self, user_id: &UserId) -> Option<&User> {
        self.users.get(user_id)
    }

    /// Add or update a guild member in the cache.
    pub fn update_guild_member(&mut self, guild_id: &GuildId, user_id: &UserId, member: &GuildMember) {
        if let Some(guild) = self.guilds.get_mut(guild_id) {
            if let Some(cache_member) = guild.members.get_mut(user_id) {
                cache_member.update(member)
            } else {
                guild.members.insert(user_id.clone(), member.clone());
            }
        }

        if let Some(guild_member_user) = &member.user {
            self.update_user(guild_member_user);
        }
    }

    /// Returns a reference to a guild member if it exists.
    pub fn get_guild_member(&self, guild_id: &GuildId, user_id: &UserId) -> Option<&GuildMember> {
        if let Some(guild) = self.guilds.get(guild_id) {
            guild.members.get(user_id)
        } else {
            None
        }
    }

    /// Add or update a guild role in the cache.
    pub fn update_guild_role(&mut self, guild_id: &GuildId, role: Role) {
        if let Some(guild) = self.guilds.get_mut(guild_id) {
            if let Some(cache_role) = guild.roles.get_mut(&role.id) {
                cache_role.update(&role)
            } else {
                guild.roles.insert(role.id.clone(), role.clone());
            }
        }
    }

    /// Add or update multiple guild roles in the cache.
    pub fn update_guild_roles(&mut self, guild_id: &GuildId, roles: Vec<Role>) {
        if let Some(guild) = self.guilds.get_mut(guild_id) {
            for role in roles {
                if let Some(cache_role) = guild.roles.get_mut(&role.id) {
                    cache_role.update(&role)
                } else {
                    guild.roles.insert(role.id.clone(), role.clone());
                }
            }
        }
    }

    /// Get a guild role by its id.
    pub fn get_guild_role(&self, guild_id: &GuildId, role_id: &Snowflake) -> Option<&Role> {
        if let Some(guild) = self.guilds.get(guild_id) {
            guild.roles.get(role_id)
        } else {
            None
        }
    }


    /// Returns the number of users in the cache.
    pub fn get_user_size(&self) -> usize {
        self.users.len()
    }

    /// Returns the number of channels in the cache.
    pub fn get_channel_size(&self) -> usize {
        self.channels.len()
    }

    /// Returns the number of guilds in the cache.
    pub fn get_guild_size(&self) -> usize {
        self.guilds.len()
    }
}

/// A cache structure for a specific type of item.
///
/// This structure is used to store items in a cache and to manage them.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct CacheDock<I: Hash + Eq + PartialEq + Clone, T: Clone> {
    items: HashMap<I, CacheItem<T>>,
    max_size: usize
}

impl<I: Hash + Eq + PartialEq + Clone, T: Clone> CacheDock<I, T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: HashMap::new(),
            max_size
        }
    }

    pub fn insert(&mut self, id: I, item: T) {
        self.items.insert(id, CacheItem::new(item));
    }

    pub fn get(&self, id: &I) -> Option<&T> {
        self.items.get(id).map(|item| item.get())
    }

    pub fn get_mut(&mut self, id: &I) -> Option<&mut T> {
        self.items.get_mut(id).map(|item| item.get_mut())
    }

    pub fn remove(&mut self, id: &I) -> Option<T> {
        self.items.remove(id).map(|item| item.item)
    }

    pub fn manage_size(&mut self){
        if self.items.len() <= self.max_size { return; }

        // sort by the "accessed" field
        let mut items: Vec<(&I, &CacheItem<T>)> = self.items.iter().collect();

        items.sort_by(|a, b| a.1.accessed.cmp(&b.1.accessed));

        while self.items.len() > self.max_size {
            items.remove(0);
        }

        self.items = items.into_iter().map(|(id, item)| (id.clone(), item.clone())).collect();
    }

}

/// A cache item.
///
/// This structure is used to store items in a cache and to manage them.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct CacheItem<T: Clone> {
    /// The time the item was added to the cache, in seconds.
    pub accessed: u64,
    /// The item.
    pub item: T
}

impl<T: Clone> CacheItem<T> {
    pub fn new(item: T) -> Self {
        Self {
            accessed: Instant::now().elapsed().as_secs(),
            item
        }
    }

    pub fn get(&self) -> &T {
        &self.item
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.item
    }
}