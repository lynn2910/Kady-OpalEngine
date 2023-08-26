use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, MySqlPool};
use client::models::guild::GuildId;
use client::models::Snowflake;
use client::models::user::UserId;
use error::{DatabaseError, Error, Result};
use crate::constants::GUILD_LIFETIME;

// const GUILD_QUERY: &str = r#"SELECT
//     guilds.*,
//     gcg.enabled AS ghostping_enabled,
//     gcg.channel AS ghostping_channel,
//     gcj.enabled AS join_enabled,
//     gcj.channel AS join_channel,
//     gcj.message AS join_message,
//     gcl.enabled AS leave_enabled,
//     gcl.channel AS leave_channel,
//     gcl.message AS leave_message,
//     g.enabled AS logs_enabled,
//     gcs.enabled AS suggestions_enabled,
//     gcs.channel AS suggestions_channel,
//     x.enabled AS xp_enabled,
//     x.cooldown AS xp_cooldown,
//     x.algorithm as xp_algo,
//     x.message as xp_message,
//     gc.enabled AS captcha_enabled,
//     gc.channel AS captcha_channel,
//     gc.role AS captcha_role,
//     gc.model AS captcha_model,
//     gc.level AS captcha_level,
//     ar.enabled as auto_role_enabled,
//     gcci.enabled as citation_enabled,
//     gcci.channel as citation_channel
// FROM guilds
//     LEFT OUTER JOIN guild_config_ghostping gcg on guilds.id = gcg.guild_id
//     LEFT OUTER JOIN guild_config_join gcj on guilds.id = gcj.guild_id
//     LEFT OUTER JOIN guild_config_leave gcl on guilds.id = gcl.guild_id
//     LEFT OUTER JOIN guild_config_logs g on guilds.id = g.guild_id
//     LEFT OUTER JOIN guild_config_suggestions gcs on guilds.id = gcs.guild_id
//     LEFT OUTER JOIN guild_config_xp x on guilds.id = x.guild_id
//     LEFT OUTER JOIN guild_config_captcha gc on guilds.id = gc.guild_id
//     LEFT OUTER JOIN guild_config_auto_roles ar on guilds.id = ar.guild_id
//     LEFT OUTER JOIN guild_config_citation gcci on guilds.id = gcci.guild_id
// WHERE id = ?;"#;

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Guild {
    pub id: GuildId,
    pub tos_accepted: bool,
    pub lang: String,
    pub join_threads: bool,
    pub sapphire: bool,

    // used to auto-delete the data from the database (RGPD friendly)
    /// Last time a event was received inside this guild
    pub last_seen: DateTime<Utc>,
    /// Last time the guild was edited (xp, configuration etc...)
    pub last_edited_timestamp: DateTime<Utc>,


    // xp config
    pub xp_enabled: Option<bool>,
    /// The algorithm chosen for the xp system
    pub xp_algo: Option<u64>,
    /// The cooldown in seconds
    pub xp_cooldown: Option<u64>,
    pub xp_message: Option<String>,
    pub xp_channel: Option<String>,

    // logs config
    pub logs_enabled: Option<bool>,

    // leave config
    pub leave_enabled: Option<bool>,
    pub leave_channel: Option<String>,
    pub leave_message: Option<String>,

    // join config
    pub join_enabled: Option<bool>,
    pub join_channel: Option<String>,
    pub join_message: Option<String>,

    // suggestions config
    pub suggestions_enabled: Option<bool>,
    pub suggestions_channel: Option<String>,

    // ghostping
    pub ghostping_enabled: Option<bool>,
    pub ghostping_channel: Option<String>,

    // captcha
    pub captcha_enabled: Option<bool>,
    pub captcha_channel: Option<String>,
    pub captcha_role: Option<Snowflake>,
    pub captcha_level: Option<u32>,
    pub captcha_model: Option<String>,

    // auto role
    pub auto_role_enabled: Option<bool>,

    // citations
    pub citation_enabled: Option<bool>,
    pub citation_channel: Option<String>
}

impl Guild {
    /// Get a guild from the database
    pub async fn from_pool(pool: &MySqlPool, request: &str, guild: &GuildId) -> Result<Self> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(guild.to_string());

        match query.fetch_one(pool).await {
            Ok(user) => Ok(user),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Ensure that the guild exists
    pub async fn ensure<T: ToString>(pool: &MySqlPool, request: &str, guild: T) -> Result<()> {
        let query = sqlx::query(request)
            .bind(guild.to_string());

        match pool.execute(query).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Create a new guild
    pub async fn create<T: ToString>(pool: &MySqlPool, request: &str, guild: T) -> Result<()> {
        let query = sqlx::query(request)
            .bind(guild.to_string());

        match pool.execute(query).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Ensure that the guild exists and return the guild
    pub async fn ensure_get(pool: &MySqlPool, request: &str, guild: GuildId) -> Result<Self> {
        Self::ensure(pool, request, guild.to_string()).await?;
        Self::from_pool(pool, request, &guild).await
    }

    /// Update the last seen timestamp
    pub async fn update_last_seen(pool: &MySqlPool, request: &str, guild: String) -> Result<()> {
        let query = sqlx::query(request)
            .bind(Utc::now())
            .bind(guild);

        match pool.execute(query).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Update the last edited timestamp
    pub async fn update_last_edited(pool: &MySqlPool, request: &str, guild: String) -> Result<()> {
        let query = sqlx::query(request)
            .bind(Utc::now())
            .bind(guild);

        match pool.execute(query).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Check if the guild can be deleted
    pub async fn can_be_deleted(pool: &MySqlPool, request: &str, id: GuildId) -> Result<bool> {
        let query = Self::from_pool(pool, request, &id).await?;

        // check if last_seen was 14 days ago
        let now = Utc::now().timestamp();
        let last_seen = query.last_seen.timestamp();

        Ok(now - last_seen >= GUILD_LIFETIME)
    }
}




/// Represent a guild action
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum GuildAction {
    Unknown = 0,
}

impl From<u64> for GuildAction {
    fn from(action: u64) -> Self {
        match action {
            0 => Self::Unknown,
            _ => Self::Unknown
        }
    }
}

/// Represent a log entry in the database
///
/// Any entries is deleted after 14 days (no adaptative lifetime)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GuildLog {
    /// The guild where the action was performed
    pub guild: GuildId,
    /// The author of the action
    pub author: UserId,
    /// The action that was performed
    pub action: GuildAction,
    /// The target of the action
    pub target: String,
    /// The reason of the action
    pub reason: String,
    /// The timestamp of the action
    pub timestamp: DateTime<Utc>,
}

impl GuildLog {
    /// Create a new log entry
    pub async fn create(
        pool: &MySqlPool,
        request: &str,
        guild: GuildId,
        author: UserId,
        action: GuildAction,
        target: String,
        reason: String
    ) -> Result<()>
    {
        let query = sqlx::query(request)
            .bind(guild.to_string())
            .bind(author.to_string())
            .bind(action as u8)
            .bind(target)
            .bind(reason)
            .bind(Utc::now());

        match pool.execute(query).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    pub async fn from_pool(pool: &MySqlPool, request: &str, guild: GuildId) -> Result<Vec<Self>> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(guild.to_string());

        match query.fetch_all(pool).await {
            Ok(logs) => Ok(logs),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }
}





/// Represent a channel logs
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GuildChannelLog {
    pub guild_id: GuildId,
    pub channel: String,
    pub log_type: String
}

impl GuildChannelLog {
    /// Get the channel logs from the database
    pub async fn from_pool<T: ToString>(pool: &MySqlPool, request: &str, guild: T) -> Result<Vec<Self>> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(guild.to_string())
            .fetch_all(pool)
            .await;

        match query {
            Ok(logs) => Ok(logs),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Create a new channel log
    pub async fn push<T: ToString>(pool: &MySqlPool, request: &str, guild: T, channel: T, log_type: T) -> Result<()> {
        let query = sqlx::query(request)
            .bind(guild.to_string())
            .bind(channel.to_string())
            .bind(log_type.to_string())
            .execute(pool)
            .await;

        match query {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Update a channel log
    pub async fn update<T: ToString>(pool: &MySqlPool, request: &str, guild: T, channel: T, log_type: T) -> Result<()> {
        let query = sqlx::query(request)
            .bind(channel.to_string())
            .bind(log_type.to_string())
            .bind(guild.to_string())
            .execute(pool)
            .await;

        match query {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }
}





/// Represent a guild user xp
///
/// Is kept as long as the user and the guild are in the database
/// If one of them is removed, the xp is removed too
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GuildUserXp {
    pub guild_id: GuildId,
    pub user_id: UserId,
    pub xp: u64
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserXpRank {
    pub rn: i64,
}

impl GuildUserXp {
    /// Get the user xp from the database
    pub async fn from_pool(pool: &MySqlPool, request: &str, guild: &GuildId, user: &UserId) -> Result<Self> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(guild.to_string())
            .bind(user.to_string());

        match query.fetch_one(pool).await {
            Ok(xp) => Ok(xp),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Ensure the presence of the user xp in the database
    pub async fn ensure(pool: &MySqlPool, request: &str, guild: &GuildId, user: &UserId) -> Result<()> {
        let query = sqlx::query(request)
            .bind(guild.to_string())
            .bind(user.to_string());

        match query.execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Add xp
    pub async fn add_xp(pool: &MySqlPool, request: &str, guild: &GuildId, user: &UserId, xp: u64) -> Result<()> {
        let query = sqlx::query(request)
            .bind(xp)
            .bind(guild.to_string())
            .bind(user.to_string());

        match query.execute(pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    /// Get the top 10 xp for a specific guild
    pub async fn get_top_10(pool: &MySqlPool, request: &str, guild: &GuildId) -> Result<Vec<GuildUserXp>> {
        let query = sqlx::query_as::<_, GuildUserXp>(request)
            .bind(guild.to_string());

        match query.fetch_all(pool).await {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }
}




#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GuildAutoRole {
    pub guild_id: GuildId,
    pub role_id: Snowflake
}

impl GuildAutoRole {
    pub async fn get_all(pool: &MySqlPool, request: &str, guild: &GuildId) -> Result<Vec<Self>> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(guild.to_string())
            .fetch_all(pool)
            .await;

        match query {
            Ok(roles) => Ok(roles),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }

    pub async fn get_single(pool: &MySqlPool, request: &str, guild: &GuildId) -> Result<Self> {
        let query = sqlx::query_as::<_, Self>(request)
            .bind(guild.to_string())
            .fetch_one(pool)
            .await;

        match query {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::Database(DatabaseError::QueryError(e.to_string())))
        }
    }
}