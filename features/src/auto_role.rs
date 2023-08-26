use client::models::guild::{GuildId, GuildMember};
use error::Result;
use log::error;
use client::manager::http::Http;
use database::Database;

pub async fn trigger(
    http: &Http,
    database: &Database,
    guild_id: &GuildId,
    guild_member: &GuildMember,
    guild_data: &database::model::guild::Guild
) {
    if !guild_data.auto_role_enabled.unwrap_or(false) {
        return;
    }

    let roles = match get_all_roles(database, guild_id).await {
        Ok(rs) => rs,
        Err(e) => {
            error!(target: "Runtime", "An error occured while acquiring the list of roles from the auto-role system: {e:#?}");
            return;
        }
    };

    for role in roles.iter() {
        let _ = guild_member.add_role(http, &role.role_id).await;
    }
}

async fn get_all_roles(database: &Database, guild_id: &GuildId) -> Result<Vec<database::model::guild::GuildAutoRole>> {
   let pool = database.get_pool().await;

    database::model::guild::GuildAutoRole::get_all(
        &pool,
        database.get_requests().await.guilds.auto_roles.get_all.as_str(),
        guild_id
    ).await
}