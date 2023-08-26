use std::collections::HashMap;
use std::fmt::Display;
use chrono::{DateTime, Utc};
use log::error;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use error::{Result, Error, EventError, ModelError, RuntimeError};
use crate::manager::cache::UpdateCache;
use crate::manager::http::{ApiResult, Http, HttpRessource};
use crate::models::Snowflake;
use crate::models::user::{User, UserId};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, sqlx::FromRow, sqlx::Decode)]
pub struct GuildId(pub Snowflake);

impl sqlx::Type<sqlx::MySql> for GuildId {
    fn type_info() -> sqlx::mysql::MySqlTypeInfo {
        <Snowflake as sqlx::Type<sqlx::MySql>>::type_info()
    }
}
impl From<String> for GuildId {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}
impl From<&str> for GuildId {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<GuildId> for String {
    fn from(value: GuildId) -> Self {
        value.0.to_string()
    }
}

impl Display for GuildId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl HttpRessource for GuildId {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        match raw.as_str() {
            Some(snowflake) => Ok(Self(snowflake.into())),
            None => Err(Error::Model(ModelError::InvalidPayload("Failed to parse guild id".into())))
        }
    }
}

/// Represents a guild that the client is in
///
/// Reference:
/// https://discord.com/developers/docs/resources/guild#guild-object
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Guild {
    /// The guild id
    pub id: GuildId,
    /// The approximate number of members in this guild
    pub member_count: u64,
    /// The name of the guild
    pub name: String,
    /// The icon hash of the guild
    pub icon: Option<String>,
    /// The owner id of the guild
    pub owner_id: String,
    /// permissions for the current user in the guild
    pub permissions: Option<u64>,
    /// The afk channel id of the guild
    pub afk_channel_id: Option<String>,
    /// The afk timeout of the guild
    pub afk_timeout: u64,
    /// The verification level of the guild
    pub verification_level: u64,
    /// The default message notification level of the guild
    pub default_message_notifications: u64,
    /// The features of the guild
    pub features: Vec<String>,
    /// The system channel id of the guild
    pub system_channel_id: Option<String>,
    /// The list of members in the guild
    pub members: HashMap<UserId, GuildMember>,
    /// The list of roles in the guild
    pub roles: HashMap<Snowflake, Role>,

    /// The shard id
    pub shard: Option<u64>,
}

impl UpdateCache for Guild {
    fn update(&mut self, from: &Self) {
        if self.member_count != from.member_count { self.member_count = from.member_count; }
        if self.name != from.name { self.name = from.name.clone(); }
        if self.icon != from.icon { self.icon = from.icon.clone(); }
        if self.owner_id != from.owner_id { self.owner_id = from.owner_id.clone(); }
        if self.permissions != from.permissions { self.permissions = from.permissions; }
        if self.afk_channel_id != from.afk_channel_id { self.afk_channel_id = from.afk_channel_id.clone(); }
        if self.afk_timeout != from.afk_timeout { self.afk_timeout = from.afk_timeout; }
        if self.verification_level != from.verification_level { self.verification_level = from.verification_level; }
        if self.default_message_notifications != from.default_message_notifications { self.default_message_notifications = from.default_message_notifications; }
        if self.features != from.features { self.features = from.features.clone(); }
        if self.system_channel_id != from.system_channel_id { self.system_channel_id = from.system_channel_id.clone(); }

        // update members
        for (id, member) in from.members.iter() {
            if !self.members.contains_key(id) {
                self.members.insert(id.clone(), member.clone());
            } else if let Some(cache_member) = self.members.get_mut(id) {
                cache_member.update(member);
            }
        }

        // update roles
        for (id, role) in from.roles.iter() {
            if !self.roles.contains_key(id) {
                self.roles.insert(id.clone(), role.clone());
            } else if let Some(cache_role) = self.roles.get_mut(id) {
                cache_role.update(role);
            }
        }
    }
}

impl HttpRessource for Guild {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = if let Some(id) = raw["id"].as_str() { id.into() } else { return Err(Error::Event(EventError::MissingField("The field 'id' is missing".to_string()))); };
        let member_count = if let Some(member_count) = raw["member_count"].as_u64() { member_count } else { 0 };
        let name = if let Some(name) = raw["name"].as_str() { name.to_string() } else { return Err(Error::Event(EventError::MissingField("The field 'name' is missing".to_string()))); };
        let icon = raw["icon"].as_str().map(|s| s.to_string());
        let owner_id = if let Some(owner_id) = raw["owner_id"].as_str() { owner_id.to_string() } else { return Err(Error::Event(EventError::MissingField("The field 'owner_id' is missing".to_string()))); };
        let permissions = raw["permissions"].as_u64();
        let afk_channel_id = raw["afk_channel_id"].as_str().map(|s| s.to_string());
        let afk_timeout = if let Some(afk_timeout) = raw["afk_timeout"].as_u64() { afk_timeout } else { return Err(Error::Event(EventError::MissingField("The field 'afk_timeout' is missing".to_string()))); };
        let verification_level = if let Some(verification_level) = raw["verification_level"].as_u64() { verification_level } else { return Err(Error::Event(EventError::MissingField("The field 'verification_level' is missing".to_string()))); };
        let default_message_notifications = if let Some(default_message_notifications) = raw["default_message_notifications"].as_u64() { default_message_notifications } else { return Err(Error::Event(EventError::MissingField("The field 'default_message_notifications' is missing".to_string()))); };
        let features = if let Some(features) = raw["features"].as_array() { features.iter().map(|v| v.as_str().unwrap().to_string()).collect() } else { return Err(Error::Event(EventError::MissingField("The field 'features' is missing".to_string()))); };
        let system_channel_id = raw["system_channel_id"].as_str().map(|s| s.to_string());

        Ok(Self {
            id,
            member_count,
            name,
            icon,
            owner_id,
            permissions,
            afk_channel_id,
            afk_timeout,
            verification_level,
            default_message_notifications,
            features,
            system_channel_id,
            shard,
            members: HashMap::new(),
            roles: HashMap::new(),
        })
    }
}

impl Guild {
    pub fn icon_url(&self, size: usize, dynamic: bool, extension: impl Display) -> Option<String> {
        self.icon.as_ref()?;

        let hash = self.icon.clone().unwrap_or("png".to_string());


        let mut extension = extension.to_string();
        if dynamic && hash.starts_with("a_") {
            extension = "gif".to_string()
        }

        Some(
            format!(
                "https://cdn.discordapp.com/icons/{id}/{hash}.{extension}?size={size}",
                id = self.id,
                hash = hash
            )
        )
    }
}

/// Represents an unavailable guild that the client is in
pub struct UnavailableGuild {
    pub id: GuildId,
    pub shard: Option<u64>
}

impl HttpRessource for UnavailableGuild {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = if let Some(id) = raw["id"].as_str() { id.into() } else { return Err(Error::Event(EventError::MissingField("The field 'id' is missing".to_string()))); };

        Ok(Self { id, shard })
    }
}


/// Represents a guild member
///
/// Reference:
/// - [Guild Member](https://discord.com/developers/docs/resources/guild#guild-member-object)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GuildMember {
    /// This field won't be included in the member object attached to MESSAGE_CREATE and MESSAGE_UPDATE gateway events.
    pub user: Option<User>,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub roles: Vec<Snowflake>,
    pub joined_at: DateTime<Utc>,
    pub premium_since: Option<DateTime<Utc>>,
    pub flags: u64,
    /// whether the user has not yet passed the guild's Membership Screening requirements
    pub pending: bool,
    // TODO: permissions
    pub permissions: Option<String>,
    pub communication_disabled_until: Option<DateTime<Utc>>,
    pub guild_id: Option<GuildId>,
}

impl UpdateCache for GuildMember {
    fn update(&mut self, from: &Self) {
        // update user
        if self.user.is_none() && from.user.is_some() {
            self.user = from.user.clone();
        } else if self.user.is_some() && from.user.is_some() {
            self.user.as_mut().unwrap().update(from.user.as_ref().unwrap());
        }

        if self.roles != from.roles { self.roles = from.roles.clone() };
        if self.nickname != from.nickname { self.nickname = from.nickname.clone(); }
        if self.avatar != from.avatar { self.avatar = from.avatar.clone(); }
        if self.joined_at != from.joined_at { self.joined_at = from.joined_at; }
        if self.premium_since != from.premium_since { self.premium_since = from.premium_since; }
        if self.flags != from.flags { self.flags = from.flags; }
        if self.pending != from.pending { self.pending = from.pending; }
        if self.permissions != from.permissions { self.permissions = from.permissions.clone(); }
        if self.communication_disabled_until != from.communication_disabled_until { self.communication_disabled_until = from.communication_disabled_until; }
    }
}

impl HttpRessource for GuildMember {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let guild_id = if let Some(guild_id) = raw.get("guild_id") { Some(GuildId::from_raw(guild_id.clone(), shard)?) } else { None };
        let user = if let Some(user) = raw["user"].as_object() { Some(User::from_raw(Value::Object(user.clone()), shard)?) } else { None };
        let nickname = raw["nick"].as_str().map(|s| s.to_string());
        let avatar = raw["avatar"].as_str().map(|s| s.to_string());
        let joined_at = if let Some(joined_at) = raw["joined_at"].as_str() { DateTime::parse_from_rfc3339(joined_at).unwrap().with_timezone(&Utc) } else { return Err(Error::Event(EventError::MissingField("The field 'joined_at' is missing".to_string()))); };
        let premium_since = raw["premium_since"].as_str().map(|s| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc));
        let flags = if let Some(flags) = raw["flags"].as_u64() { flags } else { return Err(Error::Event(EventError::MissingField("The field 'flags' is missing".to_string()))); };
        let pending = if let Some(pending) = raw["pending"].as_bool() { pending } else { return Err(Error::Event(EventError::MissingField("The field 'pending' is missing".to_string()))); };
        let permissions = raw["permissions"].as_str().map(|s| s.to_string());
        let communication_disabled_until = raw["communication_disabled_until"].as_str().map(|s| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc));
        let roles = if let Some(roles) = raw.get("roles") {
            if let Some(roles) = roles.as_array() {
                let mut collected_roles = Vec::new();

                for raw_role in roles.iter() {
                    match Snowflake::from_raw(raw_role.clone(), shard) {
                        Ok(r) => collected_roles.push(r),
                        Err(e) => { error!(target: "Conversion", "Cannot convert a role while treating a GuildMember JSON: {e:#?}"); }
                    };
                }

                collected_roles
            } else {
                return Err(Error::Event(EventError::MissingField("The field 'roles' is missing".to_string())));
            }
        } else {
            return Err(Error::Event(EventError::MissingField("The field 'roles' is missing".to_string())));
        };

        Ok(Self {
            user,
            nickname,
            avatar,
            joined_at,
            premium_since,
            flags,
            pending,
            permissions,
            communication_disabled_until,
            roles,
            guild_id: guild_id.clone(),
        })
    }
}

impl GuildMember {
    /// Adds a role to the member
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Event`] if the member has no id
    /// Returns an [`Error::Http`] if the request fails
    pub async fn add_role(&self, http: &Http, role: impl Into<Snowflake>) -> Result<ApiResult<()>> {
        if self.guild_id.is_none() {
            return Err(Error::Runtime(RuntimeError::new("No guild_id was defined")))
        }

        if let Some(user) = &self.user {
            return http.add_role_to_member(self.guild_id.as_ref().unwrap(), &user.id, &role.into()).await;
        }

        Err(Error::Event(EventError::Runtime("The GuildMember has no id".to_string())))
    }

    /// Remove a role to the member
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Event`] if the member has no id
    /// Returns an [`Error::Http`] if the request fails
    pub async fn remove_role(&self, http: &Http, role: impl Into<Snowflake>) -> Result<ApiResult<()>> {
        if self.guild_id.is_none() {
            return Err(Error::Runtime(RuntimeError::new("No guild_id was defined")))
        }

        if let Some(user) = &self.user {
            return http.remove_role_to_member(self.guild_id.as_ref().unwrap(), &user.id, &role.into()).await;
        }

        Err(Error::Event(EventError::Runtime("The GuildMember has no id".to_string())))
    }

    pub fn avatar_url(&self, size: usize, dynamic: bool, extension: impl Display) -> Option<String> {
        self.avatar.as_ref()?;

        let hash = self.avatar.clone().unwrap_or("png".to_string());


        let mut extension = extension.to_string();
        if dynamic && hash.starts_with("a_") {
            extension = "gif".to_string()
        }

        let id = match &self.user {
            Some(u) => u.id.clone(),
            None => "0".into()
        };

        Some(
            format!(
                "https://cdn.discordapp.com/avatars/{id}/{hash}.{extension}?size={size}",
                hash = hash
            )
        )
    }
}



/// Represents a role in a guild
///
/// Reference:
/// - [Role](https://discord.com/developers/docs/topics/permissions#role-object)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Role {
    pub id: Snowflake,
    pub name: String,
    pub color: u64,
    /// If this role is pinned in the user listing
    pub hoist: bool,
    pub icon: Option<String>,
    pub unicode_emoji: Option<String>,
    pub position: u64,
    // TODO: permissions
    pub permissions: String,
    /// Whether this role is managed by an integration
    pub managed: bool,
    /// Whether this role is mentionable
    pub mentionable: bool,
    /// The tags this role has
    pub tags: Option<RoleTags>
}

impl UpdateCache for Role {
    fn update(&mut self, from: &Self) {
        if self.name != from.name { self.name = from.name.clone(); }
        if self.color != from.color { self.color = from.color; }
        if self.hoist != from.hoist { self.hoist = from.hoist; }
        if self.icon != from.icon { self.icon = from.icon.clone(); }
        if self.unicode_emoji != from.unicode_emoji { self.unicode_emoji = from.unicode_emoji.clone(); }
        if self.position != from.position { self.position = from.position; }
        if self.permissions != from.permissions { self.permissions = from.permissions.clone(); }
        if self.managed != from.managed { self.managed = from.managed; }
        if self.mentionable != from.mentionable { self.mentionable = from.mentionable; }
        if self.tags != from.tags { self.tags = from.tags.clone(); }
    }
}

impl HttpRessource for Role {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let id = if let Some(id) = raw["id"].as_str() { id.into() } else { return Err(Error::Event(EventError::MissingField(format!("The field 'id' is missing from the role's JSON, raw: {raw:#?}")))); };
        let name = if let Some(name) = raw["name"].as_str() { name.to_string() } else { return Err(Error::Event(EventError::MissingField(format!("The field 'name' is missing from the role's JSON, raw: {raw:#?}")))); };
        let color = if let Some(color) = raw["color"].as_u64() { color } else { return Err(Error::Event(EventError::MissingField(format!("The field 'color' is missing from the role's JSON, raw: {raw:#?}")))); };
        let hoist = if let Some(hoist) = raw["hoist"].as_bool() { hoist } else { return Err(Error::Event(EventError::MissingField(format!("The field 'hoist' is missing from the role's JSON, raw: {raw:#?}")))); };
        let icon = raw["icon"].as_str().map(|s| s.to_string());
        let unicode_emoji = raw["unicode_emoji"].as_str().map(|s| s.to_string());
        let position = if let Some(position) = raw["position"].as_u64() { position } else { return Err(Error::Event(EventError::MissingField(format!("The field 'position' is missing from the role's JSON, raw: {raw:#?}")))); };
        let permissions = if let Some(permissions) = raw["permissions"].as_str() { permissions.to_string() } else { return Err(Error::Event(EventError::MissingField(format!("The field 'permissions' is missing from the role's JSON, raw: {raw:#?}")))); };
        let managed = if let Some(managed) = raw["managed"].as_bool() { managed } else { return Err(Error::Event(EventError::MissingField(format!("The field 'managed' is missing from the role's JSON, raw: {raw:#?}")))); };
        let mentionable = if let Some(mentionable) = raw["mentionable"].as_bool() { mentionable } else { return Err(Error::Event(EventError::MissingField(format!("The field 'mentionable' is missing from the role's JSON, raw: {raw:#?}")))); };
        let tags = if let Some(tags) = raw.get("tags") { Some(RoleTags::from_raw(tags.clone(), shard)?) } else { None };

        Ok(Self {
            id,
            name,
            color,
            hoist,
            icon,
            unicode_emoji,
            position,
            permissions,
            managed,
            mentionable,
            tags
        })
    }
}

/// Represents the tags a role has
///
/// Reference:
/// - [Role Tags](https://discord.com/developers/docs/topics/permissions#role-object-role-tags-structure)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RoleTags {
    /// the id of the bot this role belongs to
    pub bot_id: Option<Snowflake>,
    /// the id of the integration this role belongs to
    pub integration_id: Option<Snowflake>,
    /// whether this is the guild's Booster role
    pub premium_subscriber: Option<()>,
    /// the id of this role's subscription sku and listing
    pub subscription_listing_id: Option<Snowflake>,
    /// whether this role is available for purchase
    pub available_for_purchase: Option<()>,
    /// whether this role is a guild's linked role
    pub guild_connection: Option<()>
}

impl UpdateCache for RoleTags {
    fn update(&mut self, from: &Self) {
        if self.bot_id != from.bot_id { self.bot_id = from.bot_id.clone(); }
        if self.integration_id != from.integration_id { self.integration_id = from.integration_id.clone(); }
        if self.premium_subscriber != from.premium_subscriber { self.premium_subscriber = from.premium_subscriber; }
        if self.subscription_listing_id != from.subscription_listing_id { self.subscription_listing_id = from.subscription_listing_id.clone(); }
        if self.available_for_purchase != from.available_for_purchase { self.available_for_purchase = from.available_for_purchase; }
        if self.guild_connection != from.guild_connection { self.guild_connection = from.guild_connection; }
    }
}

impl HttpRessource for RoleTags {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        let bot_id = if let Some(bot_id) = raw.get("bot_id") { Some(Snowflake::from_raw(bot_id.clone(), shard)?) } else { None };
        let integration_id = if let Some(integration_id) = raw.get("integration_id") { Some(Snowflake::from_raw(integration_id.clone(), shard)?) } else { None };
        let premium_subscriber = if raw.get("premium_subscriber").is_some() { Some(()) } else { None };
        let subscription_listing_id = if let Some(subscription_listing_id) = raw.get("subscription_listing_id") { Some(Snowflake::from_raw(subscription_listing_id.clone(), shard)?) } else { None };
        let available_for_purchase = if raw.get("available_for_purchase").is_some() { Some(()) } else { None };
        let guild_connection = if raw.get("guild_connection").is_some() { Some(()) } else { None };

        Ok(Self {
            bot_id,
            integration_id,
            premium_subscriber,
            subscription_listing_id,
            available_for_purchase,
            guild_connection
        })
    }
}