use std::fmt::Display;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use error::Result;
use crate::manager::cache::UpdateCache;
use crate::manager::http::{ApiResult, Http};
use crate::models::message::{Message, MessageBuilder};
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

impl From<User> for UserId {
    fn from(user: User) -> Self {
        user.id
    }
}

impl From<&User> for UserId {
    fn from(user: &User) -> Self {
        user.id.clone()
    }
}

impl UserId {
    pub async fn send(
        &self,
        http: &Http,
        payload: MessageBuilder
    ) -> Result<ApiResult<Message>> {
        http.send_user(self, payload).await
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
    pub public: Option<bool>,
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

impl Application {
    pub fn icon_url(&self, size: usize, extension: impl Display) -> Option<String> {
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
    pub discriminator: Option<String>,
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

    pub fn banner_url(&self, size: usize, dynamic: bool, extension: impl Display) -> Option<String> {
        self.banner.as_ref()?;

        let hash = self.banner.clone().unwrap_or("png".to_string());

        let mut extension = extension.to_string();
        if dynamic && hash.starts_with("a_") {
            extension = "gif".to_string()
        }

        Some(
            format!(
                "https://cdn.discordapp.com/banners/{id}/{hash}.{extension}?size={size}",
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum NitroType {
    None = 0,
    NitroClassic = 1,
    Nitro = 2,
    NitroBasic = 3
}

impl Serialize for NitroType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            Self::None => 0,
            Self::NitroClassic => 1,
            Self::Nitro => 2,
            Self::NitroBasic => 3
        };

        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NitroType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
        let value: i64 = Deserialize::deserialize(deserializer)?;

        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::NitroClassic),
            2 => Ok(Self::Nitro),
            3 => Ok(Self::NitroBasic),
            _ => Err(serde::de::Error::custom(format!("Unknown nitro type: {}", value)))
        }
    }
}