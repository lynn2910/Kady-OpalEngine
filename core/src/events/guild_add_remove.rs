use std::ops::Deref;
use chrono::Utc;
use log::{error, warn};
use client::manager::events::Context;
use client::models::channel::ChannelId;
use client::models::components::Color;
use client::models::components::embed::{Author, Embed, Thumbnail};
use client::models::components::message_components::{ActionRow, Button, ButtonStyle, Component};
use client::models::events::{GuildCreate, GuildDelete};
use client::models::guild::Guild;
use client::models::message::MessageBuilder;
use config::Config;
use database::Database;
use crate::constants::DEFAULT_AVATAR;
use crate::scripts::{get_client_user, get_user};

pub async fn guild_create(ctx: &Context, payload: GuildCreate) {
    if payload.guild.is_none() { return; }
    let guild = payload.guild.unwrap();

    // delete the guild from the database
    if let Some(db) = ctx.get_data::<Database>().await {
        let pool = db.get_pool().await;
        let requests = db.get_requests().await;

        let g_data = database::model::guild::Guild::get_optional(
            &pool,
            requests.guilds.get.as_str(),
            &guild.id
        ).await;

        // if Ok & None, the guild wasn't registered, so we can say that this is a new guild
        if let Ok(guild_data) = g_data {
            if guild_data.is_none() {
                // register the database
                let _ = sqlx::query(requests.guilds.create.as_str())
                    .bind(guild.id.to_string())
                    .execute(pool.deref())
                    .await;

                // send the message in the guild
                if let Some(config) = ctx.get_data::<Config>().await {
                    send_new_guild_message(
                        ctx,
                        &guild,
                        &config
                    ).await;
                } else {
                    warn!(target: "GuildCreate", "No config in the context data")
                }
            }
        }
    } else {
        warn!(target: "Runtime", "No Database was found in the context data");
    }
}

pub async fn guild_remove(ctx: &Context, payload: GuildDelete, guild: Option<Guild>) {
    if payload.unavailable { return };

    // delete the guild from the database
    if let Some(db) = ctx.get_data::<Database>().await {
        let pool = db.get_pool().await;
        let requests = db.get_requests().await;

        let res = sqlx::query(requests.guilds.delete.as_str())
            .bind(payload.id.to_string())
            .execute(pool.deref())
            .await;

        if let Err(e) = res {
            error!(target: "Runtime", "An error occured while deleting a Guild data's (bot removed): {e:#?}");
        }
    }

    // send the message in the guild
    if let Some(config) = ctx.get_data::<Config>().await {
        send_deleted_guild_message(
            ctx,
            &guild,
            &config
        ).await;
    } else {
        warn!(target: "GuildDelete", "No config in the context data")
    }
}

pub(crate) async fn send_new_guild_message(
    ctx: &Context,
    guild: &Guild,
    config: &Config
)
{
    if config.client.guild_add_channel.is_none() { return; }

    let channel_id: ChannelId = config.client.guild_add_channel.clone().unwrap().into();
    let guild_count = {
        let cache = ctx.cache.read().await;
        cache.get_guild_size()
    };
    let client_user = get_client_user(ctx).await;
    let owner = match get_user(ctx, &guild.owner_id.clone().into()).await {
        Some(owner) => owner.global_name.unwrap_or(owner.username),
        None => guild.owner_id.to_string()
    };

    let _ = channel_id.send_message(
        &ctx.skynet,
        MessageBuilder::new()
            .add_embed(
                Embed::new()
                    .set_thumbnail(
                        Thumbnail::new(
                            guild.icon_url(512, false, "png")
                                .unwrap_or(DEFAULT_AVATAR.to_string())
                        )
                    )
                    .set_author(
                        Author::new()
                            .set_name(
                                client_user.as_ref()
                                    .map(|app|
                                        app.global_name.clone()
                                            .unwrap_or(app.username.clone())
                                    )
                                    .unwrap_or("Skynet".to_string())
                            )
                            .set_icon_url(
                                Some(
                                    client_user.map(|app|
                                        app.avatar_url(512, false, "png").unwrap_or(DEFAULT_AVATAR.to_string()))
                                        .unwrap_or(DEFAULT_AVATAR.to_string())
                                )
                            )
                    )
                    .set_timestamp(Utc::now())
                    .set_title("Un serveur a rejoint cette aventure")
                    .set_color(Color::from_rgb(84, 223, 118))
                    .set_description(format!("💖 Le serveur **{}** appartient à **{owner}**.\n\n> Ce serveur possède **{}** membres incroyables\n\n✨ **Bienvenue dans cette grande famille !**", guild.name, guild.member_count))
                    .set_thumbnail(
                        Thumbnail::new(
                            guild.icon_url(512, false, "png")
                                .unwrap_or(DEFAULT_AVATAR.to_string())
                        )
                    )
            )
            .add_component(
                Component::ActionRow(
                    ActionRow::new()
                        .add_component(
                            Component::Button(
                                Button::new("A")
                                    .set_disabled(true)
                                    .set_style(ButtonStyle::Secondary)
                                    .set_label(format!("Je suis désormais sur {guild_count} serveurs"))
                            )
                        )
                )
            )
    ).await;
}

pub(crate) async fn send_deleted_guild_message(
    ctx: &Context,
    guild: &Option<Guild>,
    config: &Config
)
{
    if config.client.guild_remove_channel.is_none() { return; }

    let channel_id: ChannelId = config.client.guild_remove_channel.clone().unwrap().into();
    let guild_count = {
        let cache = ctx.cache.read().await;
        cache.get_guild_size()
    };
    let client_user = get_client_user(ctx).await;

    let owner = {
        if let Some(g) = guild {
            match get_user(ctx, &g.owner_id.clone().into()).await {
                Some(owner) => owner.global_name.unwrap_or(owner.username),
                None => g.owner_id.to_string()
            }
        } else {
            "Unknown".into()
        }
    };

    let _ = channel_id.send_message(
        &ctx.skynet,
        MessageBuilder::new()
            .add_embed(
                Embed::new()
                    .set_title("Un serveur a quitté l'aventure...")
                    .set_color(Color::from_rgb(223, 84, 84))
                    .set_thumbnail(
                        Thumbnail::new(
                            if let Some(g) = guild {
                                g.icon_url(512, false, "png")
                                    .unwrap_or(DEFAULT_AVATAR.to_string())
                            } else {
                                DEFAULT_AVATAR.to_string()
                            }
                        )
                    )
                    .set_author(
                        Author::new()
                            .set_name(
                                client_user.as_ref()
                                    .map(|app|
                                        app.global_name.clone().unwrap_or(app.username.clone()))
                                    .unwrap_or("Skynet ".to_string())
                            )
                            .set_icon_url(
                                Some(
                                    client_user.map(|app|
                                        app.avatar_url(512, false, "png")
                                            .unwrap_or(DEFAULT_AVATAR.to_string())
                                    )
                                        .unwrap_or(DEFAULT_AVATAR.to_string())
                                )
                            )
                    )
                    .set_timestamp(Utc::now())
                    .set_description(
                        format!(
                            "💔 Le serveur **{}** a quitté le navire\n\n> Il appartenait à **{owner}**\n\n> **✨ Bonne continuation à eux !**",
                            if let Some(g) = guild { g.name.clone() } else { "Unknown".to_string() }
                        )
                    )
            )
            .add_component(
                Component::ActionRow(
                    ActionRow::new()
                        .add_component(
                            Component::Button(
                                Button::new("A")
                                    .set_disabled(true)
                                    .set_style(ButtonStyle::Secondary)
                                    .set_label(format!("Je suis désormais sur {guild_count} serveurs"))
                            )
                        )
                )
            )
    ).await;
}