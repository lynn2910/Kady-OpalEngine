use log::error;
use client::manager::events::Context;
use client::models::events::GuildMemberAdd;
use client::models::guild::Role;
use database::{Database, model};

pub(crate) async fn triggered(ctx: &Context, payload: GuildMemberAdd) {
    let database = ctx.get_data::<Database>().await.expect("No database found");

    let guild_data = {
        let pool = database.get_pool().await;
        match model::guild::Guild::from_pool(&pool, database.get_requests().await.guilds.get.as_str(), &payload.guild_id).await {
            Ok(guild_data) => guild_data,
            Err(e) => {
                error!("Error while fetching guild data: {:?}", e);
                return;
            }
        }
    };

    let captcha_task = captcha(ctx, &payload, &guild_data);

    // Wait for all async functions to finish before returning
    captcha_task.await;
}

async fn captcha(ctx: &Context, payload: &GuildMemberAdd, guild_data: &model::guild::Guild){
    // we check if:
    // - captcha is enabled
    // - captcha channel is set
    // - captcha role is set
    if !guild_data.captcha_enabled.unwrap_or(false) || guild_data.captcha_channel.is_none() || guild_data.captcha_role.is_none() {
        let database = ctx.get_data::<Database>().await.expect("No database found");

        // if the captcha system is off, we trigger immediately the auto-role system
        features::auto_role::trigger(
            &ctx.skynet,
            &database,
            &payload.guild_id,
            &payload.member,
            guild_data
        ).await;
        // then we stop this function
        return;
    }

    // we check if the role of the captcha exist
    let captcha_role = if let Some(role) = get_captcha_role(ctx, payload, guild_data).await { role } else { return; };

    if let Err(e) = payload.member.add_role(&ctx.skynet, &captcha_role.id).await {
        error!("Error while adding role to member: {:?}", e);
    }
}

/// Get the captcha role from the cache or from the API
async fn get_captcha_role(
    ctx: &Context,
    payload: &GuildMemberAdd,
    guild_data: &model::guild::Guild
) -> Option<Role>
{
    let id = if let Some(id) = &guild_data.captcha_role { id.clone() } else { return None; };

    // firstly, we check in the cache
    {
        let cache = ctx.cache.read().await;
        if let Some(role) = cache.get_guild_role(&payload.guild_id, &id) {
            return Some(role.clone());
        }
    }

    // if the role is not in the cache, we fetch it from the API
    let roles = match ctx.skynet.fetch_guild_roles(&payload.guild_id).await {
        Ok(Ok(role)) => role,
        Ok(Err(e)) => {
            error!("Error from the api while fetching role: {:?}", e);
            return None;
        }
        Err(e) => {
            error!("Error while fetching role: {:?}", e);
            return None;
        }
    };

    // we insert the role in the cache
    {
        let mut cache = ctx.cache.write().await;
        cache.update_guild_roles(&payload.guild_id, roles.clone());
    }

    // we now re-check from the cache
    {
        let cache = ctx.cache.read().await;
        if let Some(role) = cache.get_guild_role(&payload.guild_id, &id) {
            return Some(role.clone());
        }
    }

    None
}