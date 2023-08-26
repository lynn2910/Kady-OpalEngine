use std::path::PathBuf;
use serde::{Serialize, Deserialize };
use error::{DatabaseError, Error, FileError, Result};

#[derive(Serialize, Deserialize, Debug)]
pub struct DynamicRequest {
    pub users: UserRequests,
    pub guilds: GuildRequests,
}





/// Contain all requests for the users table and its related tables
#[derive(Serialize, Deserialize, Debug)]
pub struct UserRequests {
    pub get: String,
    pub ensure: String,
    pub create: String,
    pub ensure_get: String,
    pub update_last_seen: String,
    pub update_last_edited_timestamp: String,
    pub marriage: UserMarriage,
    pub reputation: UserReputation,
}

/// Contain all requests for the marriage table
#[derive(Serialize, Deserialize, Debug)]
pub struct UserMarriage {
    pub get: String
}

/// Contain all requests for the reputation table
#[derive(Serialize, Deserialize, Debug)]
pub struct UserReputation {
    pub get: String,
    pub get_guild: String,
    pub get_last: String,
    pub get_last_guild: String,
    pub get_top_10_global: String,
    pub get_top_10_guild: String,
    pub get_user_rank_global: String,
    pub get_user_rank_guild: String,
}







/// Contain all requests for the guilds table and its related tables
#[derive(Serialize, Deserialize, Debug)]
pub struct GuildRequests {
    pub get: String,
    pub ensure: String,
    pub create: String,
    pub ensure_get: String,
    pub update_last_seen: String,
    pub update_last_edited_timestamp: String,
    pub logs: GuildLogs,
    pub channel_logs: GuildChannelLogs,
    pub xp: GuildXp,
    pub auto_roles: GuildAutoRole
}

/// Contain all requests for the guild_logs table
#[derive(Serialize, Deserialize, Debug)]
pub struct GuildLogs {
    pub get: String,
    pub create: String,
}

/// Contain all requests for the guild_channel_logs table
#[derive(Serialize, Deserialize, Debug)]
pub struct GuildChannelLogs {
    pub get_all: String,
    pub get_by_type: String,
    pub push: String,
    pub update: String,
}

/// Contain all requests for the guild_xp table
#[derive(Serialize, Deserialize, Debug)]
pub struct GuildXp {
    pub get: String,
    pub add_xp: String,
    pub ensure: String,
    pub get_top_10: String,
    pub get_rank: String
}

/// Contain all requests for the guild_auto_roles table
#[derive(Serialize, Deserialize, Debug)]
pub struct GuildAutoRole {
    pub get_all: String,
    pub get_single: String
}




impl DynamicRequest {
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return Err(Error::Fs(FileError::CannotReadFile(e.to_string())))
        };

        match toml::from_str(&content) {
            Ok(d) => Ok(d),
            Err(e) => Err(Error::Database(DatabaseError::CannotParseDynamicRequestTable(e.to_string())))
        }
    }
}