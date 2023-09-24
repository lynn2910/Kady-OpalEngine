//! Unique error ID for this file: 11xxx

use std::str::FromStr;
use log::error;
use uuid::Uuid;
use client::manager::events::Context;
use client::models::channel::Channel;
use client::models::events::InteractionCreate;
use client::models::message::MessageBuilder;
use client::models::Snowflake;
use database::Database;
use features::captcha;
use translation::{ message, fmt::formatter::Formatter };
use crate::broadcast_error;
use crate::crates::error_broadcaster::*;
use crate::scripts::{get_guild_locale, QueryParams};

pub(in crate::scripts) async fn triggered(ctx: &Context, payload: &InteractionCreate, query: QueryParams) {
    let guild_id = match &payload.interaction.guild_id {
        Some(id) => id.clone(),
        _ => return
    };


    // verify the integrity of the query
    // we are getting 'code' and 'instance'
    let query_code = match query.get("code") {
        Some(c) => c,
        None => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            get_guild_locale(&payload.interaction.guild_locale),
                            "errors::internal_error",
                            Formatter::new().add("code", "11001")
                        )
                    )
                    .set_ephemeral(true)
            ).await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(Some(guild_id))
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/buttons/captcha_try.rs:triggered:47"),
                interaction: BroadcastInteraction::default()
                    .set_name("captcha_try")
                    .set_type(BroadcastInteractionType::Button),
                details: BroadcastDetails::default()
                    .add("code", "11001")
                    .add("error", "Cannot get the 'code' query parameter"),
                ctx.skynet.as_ref()
            );

            return;
        }
    };
    let query_instance = match query.get("instance") {
        Some(c) => c,
        None => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            get_guild_locale(&payload.interaction.guild_locale),
                            "errors::internal_error",
                            Formatter::new().add("code", "11002")
                        )
                    )
                    .set_ephemeral(true)
            ).await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(Some(guild_id))
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/buttons/captcha_try.rs:triggered:80"),
                interaction: BroadcastInteraction::default()
                    .set_name("captcha_try")
                    .set_type(BroadcastInteractionType::Button),
                details: BroadcastDetails::default()
                    .add("code", "11002")
                    .add("error", "Cannot get the 'instance' query parameter"),
                ctx.skynet.as_ref()
            );

            return;
        }
    };

    // we get the guild data
    let database = ctx.get_data::<Database>().await.expect("No database found");
    let guild_data = {
        let pool = database.get_pool().await;
        match database::model::guild::Guild::from_pool(&pool, database.get_requests().await.guilds.get.as_str(), &guild_id).await {
            Ok(guild_data) => guild_data,
            Err(e) => {
                error!("Error while fetching guild data: {:?}", e);
                cannot_get_guild_data(ctx, payload).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(Some(guild_id))
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/buttons/captcha_try.rs:triggered:108"),
                    interaction: BroadcastInteraction::default()
                        .set_name("captcha_try")
                        .set_type(BroadcastInteractionType::Button),
                    details: BroadcastDetails::default()
                        .add("error", "Cannot get the guild data"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        }
    };

    // we verify if the configuration is okay
    if guild_data.captcha_role.is_none() || guild_data.captcha_channel.is_none() {
        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(
                    message!(
                            guild_data.lang,
                            "features::captcha::invalid_config",
                            Formatter::new().add("code", "11003")
                        )
                )
                .set_ephemeral(true)
        ).await;
        return;
    }

    // get the Client data avec the captcha container
    let mut data = ctx.data.write().await;
    let captcha_container = match data.get_mut::<captcha::CaptchaContainer>() {
        Some(cc) => cc,
        _ => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            guild_data.lang,
                            "errors::internal_error",
                            Formatter::new().add("code", "11003")
                        )
                    )
                    .set_ephemeral(true)
            ).await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(Some(guild_id))
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/buttons/captcha_try.rs:triggered:155"),
                interaction: BroadcastInteraction::default()
                    .set_name("captcha_try")
                    .set_type(BroadcastInteractionType::Button),
                details: BroadcastDetails::default()
                    .add("code", "11003")
                    .add("error", "Cannot get the captcha container"),
                ctx.skynet.as_ref()
            );

            return;
        }
    };

    match captcha_container.get(Uuid::from_str(query_instance.as_str()).unwrap_or(Default::default())).await {
        Some(instance) => {
            let good_code = instance.code.clone().iter().collect::<String>();
            if &good_code == query_code {
                // OKAY
                let guild_member = match &payload.interaction.member {
                    Some(m) => m,
                    _ => return cannot_get_guild_member(ctx, payload).await
                };

                // we send the message
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(guild_data.lang.clone(), "features::captcha::ok"))
                        .set_ephemeral(true)
                ).await;

                // we call the auto-role system
                features::auto_role::trigger(
                    &ctx.skynet,
                    &database,
                    &guild_id,
                    guild_member,
                    &guild_data
                ).await;

                let role_removed = guild_member.remove_role(&ctx.skynet, guild_data.captcha_role.clone().unwrap_or(Snowflake("0".into()))).await;

                // if an error occured, we notify the user
                if let Err(e) = role_removed {
                    error!(target: "Runtime", "An error occured while adding a role within the captcha system: {e:#?}");

                    if let Some(channel) = &payload.interaction.channel {
                        let msg = MessageBuilder::new()
                            .set_content(
                                message!(
                                    guild_data.lang.clone(),
                                    "errors::cannot_remove_role",
                                    Formatter::new().add("id", guild_member.user.as_ref().unwrap().id.to_string())
                                )
                            );

                        match channel {
                            Channel::GuildText(chl) => { let _ = chl.id.send_message(&ctx.skynet, msg).await; },
                            Channel::GuildAnnouncement(chl) => { let _ = chl.id.send_message(&ctx.skynet, msg).await; },
                            _ => {}
                        }
                    }
                }

            } else {
                captcha_container.remove_instance(instance.id).await;

                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(
                            message!(
                                guild_data.lang,
                                "features::captcha::bad_code"
                            )
                        )
                        .set_ephemeral(true)
                ).await;
            }
        },
        _ => {
            // no instance were found
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            guild_data.lang,
                            "features::captcha::invalid_instance"
                        )
                    )
                    .set_ephemeral(true)
            ).await;
        }
    }
}

async fn cannot_get_guild_data(ctx: &Context, payload: &InteractionCreate) {
    let guild_locale = get_guild_locale(&payload.interaction.guild_locale);
    let _ = payload.interaction.reply(
        &ctx.skynet,
        MessageBuilder::new()
            .set_content(message!(guild_locale, "errors::cannot_get_guild_data"))
            .set_ephemeral(true)
    ).await;
}

async fn cannot_get_guild_member(ctx: &Context, payload: &InteractionCreate) {
    let guild_locale = get_guild_locale(&payload.interaction.guild_locale);
    let _ = payload.interaction.reply(
        &ctx.skynet,
        MessageBuilder::new()
            .set_content(
                message!(
                    guild_locale,
                    "errors::internal_error",
                    Formatter::new().add("code", "11004")
                )
            )
            .set_ephemeral(true)
    ).await;
}