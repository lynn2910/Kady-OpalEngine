use std::fmt::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use error::{Result, Error, EventError};
use crate::manager::cache::UpdateCache;
use crate::manager::http::HttpRessource;
use crate::models::Snowflake;

/// Represent the id of a user
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::FromRow, sqlx::Decode)]
pub struct UserId(pub Snowflake);

impl sqlx::Type<sqlx::MySql> for UserId {
    fn type_info() -> sqlx::mysql::MySqlTypeInfo {
        <Snowflake as sqlx::Type<sqlx::MySql>>::type_info()
    }
}

impl From<String> for UserId {
    fn from(id: String) -> Self {
        Self(id.into())
    }
}
impl From<&String> for UserId {
    fn from(id: &String) -> Self {
        Self(id.into())
    }
}
impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<UserId> for String {
    fn from(value: UserId) -> Self {
        value.0.to_string()
    }
}

impl From<&str> for UserId {
    fn from(id: &str) -> Self {
        Self(id.into())
    }
}

impl HttpRessource for UserId {
    fn from_raw(raw: Value, shard: Option<u64>) -> Result<Self> {
        Ok(Self(Snowflake::from_raw(raw, shard)?))
    }
}

/// Represent the user of the client
///
/// Reference:
/// - [User](https://discord.com/developers/docs/resources/user#user-object)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ClientUser {
    pub avatar: Option<String>,
    pub bot: bool,
    pub discriminator: String,
    pub flags: Option<u32>,
    pub global_name: Option<String>,
    pub id: UserId,
    pub verified: bool,
    pub username: String,
    pub mfa_enabled: bool,
}

impl UpdateCache for ClientUser {
    fn update(&mut self, from: &Self) {
        if self.verified != from.verified { self.verified = from.verified }
        if self.bot != from.bot { self.bot = from.bot }
        if self.mfa_enabled != from.mfa_enabled { self.mfa_enabled = from.mfa_enabled }
        if self.avatar != from.avatar { self.avatar = from.avatar.clone() }
        if self.discriminator != from.discriminator { self.discriminator = from.discriminator.clone() }
        if self.flags != from.flags { self.flags = from.flags }
        if self.global_name != from.global_name { self.global_name = from.global_name.clone() }
        if self.id != from.id { self.id = from.id.clone() }
        if self.username != from.username { self.username = from.username.clone() }
    }
}

impl HttpRessource for ClientUser {
    fn from_raw(raw: Value, _: Option<u64>) -> Result<Self> {
        let avatar = raw["avatar"].as_str().map(|avatar| avatar.to_string());
        let bot = if let Some(bot) = raw["bot"].as_bool() { bot } else { return Err(Error::Event(EventError::MissingField("No 'bot' field".into()))) };
        let discriminator = if let Some(discriminator) = raw["discriminator"].as_str() { discriminator.to_string() } else { return Err(Error::Event(EventError::MissingField("No 'discriminator' field".into()))) };
        let flags = raw["flags"].as_u64().map(|flags| flags as u32);
        let global_name = raw["global_name"].as_str().map(|global_name| global_name.to_string());
        let id = if let Some(id) = raw["id"].as_str() { id.to_string().into() } else { return Err(Error::Event(EventError::MissingField("No 'id' field".into()))) };
        let verified = if let Some(verified) = raw["verified"].as_bool() { verified } else { return Err(Error::Event(EventError::MissingField("No 'verified' field".into()))) };
        let username = if let Some(username) = raw["username"].as_str() { username.to_string() } else { return Err(Error::Event(EventError::MissingField("No 'username' field".into()))) };
        let mfa_enabled = if let Some(mfa_enabled) = raw["mfa_enabled"].as_bool() { mfa_enabled } else { return Err(Error::Event(EventError::MissingField("No 'mfa_enabled' field".into()))) };

        Ok(Self { avatar, bot, discriminator, flags, global_name, id, verified, username, mfa_enabled })
    }
}

impl ClientUser {
    pub fn tag(&self) -> String {
        format!("{}#{}", self.username, self.discriminator)
    }

    pub fn avatar_url(&self, size: usize, dynamic: bool, extension: impl Display) -> Option<String> {
        self.avatar.as_ref()?;

        let hash = self.avatar.clone().unwrap_or("png".to_string());


        let mut extension = extension.to_string();
        if dynamic && hash.starts_with("a_") {
            extension = "gif".to_string()
        }

        Some(
            format!(
                "https://cdn.discordapp.com/avatars/{id}/{hash}.{extension}?size={size}",
                id = self.id,
                hash = hash
            )
        )
    }
}

/// Represent the application of the client
///
/// Reference:
/// - [Application](https://discord.com/developers/docs/resources/application#application-object)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Application {
    pub flags: u64,
    pub id: Snowflake,
    pub public: bool,
    pub name: String,
    pub description: String,
    pub summary: String,
    pub cover_image: Option<String>,
}

impl UpdateCache for Application {
    fn update(&mut self, from: &Self) {
        if self.flags != from.flags { self.flags = from.flags }
        if self.id != from.id { self.id = from.id.clone() }
        if self.public != from.public { self.public = from.public }
        if self.name != from.name { self.name = from.name.clone() }
        if self.description != from.description { self.description = from.description.clone() }
        if self.summary != from.summary { self.summary = from.summary.clone() }
        if self.cover_image != from.cover_image { self.cover_image = from.cover_image.clone() }
    }
}

impl HttpRessource for Application {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        let flags = if let Some(flags) = raw["flags"].as_u64() { flags } else { return Err(Error::Event(EventError::MissingField("No 'flags' field".into()))) };
        let id = if let Some(id) = raw["id"].as_str() { id.to_string().into() } else { return Err(Error::Event(EventError::MissingField("No 'id' field".into()))) };
        let public = if let Some(public) = raw["bot_public"].as_bool() { public } else { return Err(Error::Event(EventError::MissingField("No 'bot_public' field".into()))) };
        let name = if let Some(name) = raw["name"].as_str() { name.to_string() } else { return Err(Error::Event(EventError::MissingField("No 'name' field".into()))) };
        let description = if let Some(description) = raw["description"].as_str() { description.to_string() } else { return Err(Error::Event(EventError::MissingField("No 'description' field".into()))) };
        let summary = if let Some(summary) = raw["summary"].as_str() { summary.to_string() } else { return Err(Error::Event(EventError::MissingField("No 'summary' field".into()))) };
        let cover_image = raw["cover_image"].as_str().map(|cover_image| cover_image.to_string());

        Ok(Self { flags, id, public, name, description, summary, cover_image })
    }
}

impl Application {
    pub fn icon_url(&self, size: usize, extension: impl Display) -> Option<String> {

        dbg!(&self.cover_image);

        self.cover_image.as_ref()?;

        Some(
            format!(
                "https://cdn.discordapp.com/app-icons/{id}/{hash}.{extension}?size={size}",
                id = self.id,
                hash = self.cover_image.clone().unwrap_or("png".to_string()),
                extension = extension,
                size = size
            )
        )
    }
}

/// Represent a User
///
/// Reference:
/// - [User](https://discord.com/developers/docs/resources/user#user-object)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub discriminator: String,
    /// the user's display name, if it is set. For bots, this is the application name
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
    pub system: Option<bool>,
    pub banner: Option<String>,
    pub accent_color: Option<u64>,
    pub locale: Option<String>,
    pub flags: Option<u64>,
    pub public_flags: Option<u64>,
    pub premium_type: Option<NitroType>,
}

impl User {
    pub fn avatar_url(&self, size: usize, dynamic: bool, extension: impl Display) -> Option<String> {
        self.avatar.as_ref()?;

        let hash = self.avatar.clone().unwrap_or("png".to_string());


        let mut extension = extension.to_string();
        if dynamic && hash.starts_with("a_") {
            extension = "gif".to_string()
        }

        Some(
            format!(
                "https://cdn.discordapp.com/avatars/{id}/{hash}.{extension}?size={size}",
                id = self.id,
                hash = hash
            )
        )
    }
}

impl UpdateCache for User {
    fn update(&mut self, from: &Self) {
        if self.id != from.id { self.id = from.id.clone() }
        if self.username != from.username { self.username = from.username.clone() }
        if self.discriminator != from.discriminator { self.discriminator = from.discriminator.clone() }
        if self.global_name != from.global_name { self.global_name = from.global_name.clone() }
        if self.avatar != from.avatar { self.avatar = from.avatar.clone() }
        if self.bot != from.bot { self.bot = from.bot }
        if self.system != from.system { self.system = from.system }
        if self.banner != from.banner { self.banner = from.banner.clone() }
        if self.accent_color != from.accent_color { self.accent_color = from.accent_color }
        if self.locale != from.locale { self.locale = from.locale.clone() }
        if self.flags != from.flags { self.flags = from.flags }
        if self.public_flags != from.public_flags { self.public_flags = from.public_flags }
        if self.premium_type != from.premium_type { self.premium_type = from.premium_type.clone() }
    }
}

impl HttpRessource for User {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        let id: UserId = if let Some(id) = raw["id"].as_str() { id.to_string().into() } else { return Err(Error::Event(EventError::MissingField("No 'id' field".into()))) };
        let username = if let Some(username) = raw["username"].as_str() { username.to_string() } else { return Err(Error::Event(EventError::MissingField("No 'username' field".into()))) };
        let discriminator = if let Some(discriminator) = raw["discriminator"].as_str() { discriminator.to_string() } else { return Err(Error::Event(EventError::MissingField("No 'discriminator' field".into()))) };
        let global_name = raw["global_name"].as_str().map(|global_name| global_name.to_string());
        let avatar = raw["avatar"].as_str().map(|avatar| avatar.to_string());
        let bot = raw["bot"].as_bool();
        let system = raw["system"].as_bool();
        let banner = raw["banner"].as_str().map(|banner| banner.to_string());
        let accent_color = raw["accent_color"].as_u64();
        let locale = raw["locale"].as_str().map(|locale| locale.to_string());
        let flags = raw["flags"].as_u64();
        let public_flags = raw["public_flags"].as_u64();
        let premium_type = raw["premium_type"].as_u64().map(NitroType::from_u64);

        Ok(Self { id, username, discriminator, global_name, avatar, bot, system, banner, accent_color, locale, flags, public_flags, premium_type })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum NitroType {
    None = 0,
    NitroClassic = 1,
    Nitro = 2,
    NitroBasic = 3
}

impl NitroType {
    pub(crate) fn from_u64(n: u64) -> Self {
        match n {
            1 => Self::NitroClassic,
            2 => Self::Nitro,
            3 => Self::NitroBasic,
            _ => Self::None
        }
    }
}

impl HttpRessource for NitroType {
    fn from_raw(raw: Value, _shard: Option<u64>) -> Result<Self> {
        let n = if let Some(n) = raw.as_u64() { n } else { return Err(Error::Event(EventError::MissingField("No 'premium_type' field".into()))) };

        Ok(Self::from_u64(n))
    }
}