//! Error's prefix ID: 01xxx

use chrono::Utc;
use log::error;
use client::manager::events::Context;
use client::models::components::Color;
use client::models::components::embed::{Author, Embed};
use client::models::events::InteractionCreate;
use client::models::interaction::InteractionDataOptionValue;
use client::models::message::MessageBuilder;
use database::Database;
use database::model::guild::Guild;
use translation::message;
use crate::assets;
use crate::scripts::get_guild_locale;
use crate::crates::error_broadcaster::*;
use crate::broadcast_error;
use crate::scripts::slashs::internal_error;

pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
    let local = get_guild_locale(&payload.interaction.guild_locale);

    let guild_id = match &payload.interaction.guild_id {
        Some(id) => id,
        None => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new().set_content(message!(local, "errors::not_guild"))
            ).await;
            return;
        }
    };

    let database = ctx.get_data::<Database>().await.expect("Cannot acquire the database structure");
    let guild_data = {
        let pool = database.get_pool().await;
        match Guild::from_pool(&pool, &database.get_requests().await.guilds.get, guild_id).await {
            Ok(g) => g,
            Err(e) => {
                error!(target: "Runtime", "An error occured while acquiring the guild informations: {e:#?}");
                cannot_get_guild_data(ctx, payload).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/citation.rs:47"),
                    interaction: BroadcastInteraction::default()
                        .set_name("citation")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("error", format!("{e:#?}"))
                        .add("reason", "Cannot get the guild data"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        }
    };

    if !guild_data.citation_enabled.unwrap_or(false) {
        not_enabled(ctx, payload).await;
        return
    }

    // we check if the channel exist
    {
        if let Some(id) = guild_data.citation_channel.clone() {
            match ctx.skynet.fetch_channel(&id.into()).await {
                Ok(c) => match c {
                    Ok(_) => (),
                    Err(e) => {
                        error!(target: "Runtime", "An error was received from the api while fetching the citation's channel: {e:#?}");
                        internal_error(ctx, &payload.interaction, local , "12002").await;

                        broadcast_error!(
                            localisation: BroadcastLocalisation::default()
                                .set_guild(payload.interaction.guild_id.clone())
                                .set_channel(payload.interaction.channel_id.clone())
                                .set_code_path("core/src/scripts/slashs/citation.rs:74"),
                            interaction: BroadcastInteraction::default()
                                .set_name("citation")
                                .set_type(BroadcastInteractionType::SlashCommand),
                            details: BroadcastDetails::default()
                                .add("error", format!("{e:#?}"))
                                .add("reason", "Cannot fetch the citation's channel"),
                            ctx.skynet.as_ref()
                        );

                        return;
                    }
                },
                Err(e) => {
                    error!(target: "Runtime", "An error occured while trying to fetch the citation's channel: {e:#?}");
                    internal_error(ctx, &payload.interaction, local , "12001").await;

                    broadcast_error!(
                        localisation: BroadcastLocalisation::default()
                            .set_guild(payload.interaction.guild_id.clone())
                            .set_channel(payload.interaction.channel_id.clone())
                            .set_code_path("core/src/scripts/slashs/citation.rs:103"),
                        interaction: BroadcastInteraction::default()
                            .set_name("citation")
                            .set_type(BroadcastInteractionType::SlashCommand),
                        details: BroadcastDetails::default()
                            .add("error", format!("{e:#?}"))
                            .add("reason", "Cannot fetch the citation's channel"),
                        ctx.skynet.as_ref()
                    );

                    return;
                }
            }
        } else {
            no_valid_channel(ctx, payload).await;
            return;
        }
    };

    // we get the text
    let text: String = match &payload.interaction.data {
        Some(datas) => {
            match &datas.options {
                Some(options) => {
                    if let Some(t) = &options.iter().find(|opt| opt.name == "citation") {
                        match &t.value {
                            Some(InteractionDataOptionValue::String(s)) => s.clone(),
                            _ => {
                                internal_error(ctx, &payload.interaction, local , "12006").await;

                                broadcast_error!(
                                    localisation: BroadcastLocalisation::default()
                                        .set_guild(payload.interaction.guild_id.clone())
                                        .set_channel(payload.interaction.channel_id.clone())
                                        .set_code_path("core/src/scripts/slashs/citation.rs:133"),
                                    interaction: BroadcastInteraction::default()
                                        .set_name("citation")
                                        .set_type(BroadcastInteractionType::SlashCommand),
                                    details: BroadcastDetails::default()
                                        .add("error", "Cannot get the citation's text")
                                        .add("reason", "The text is not a string"),
                                    ctx.skynet.as_ref()
                                );

                                return;
                            }
                        }
                    } else {
                        internal_error(ctx, &payload.interaction, local , "12005").await;

                        broadcast_error!(
                            localisation: BroadcastLocalisation::default()
                                .set_guild(payload.interaction.guild_id.clone())
                                .set_channel(payload.interaction.channel_id.clone())
                                .set_code_path("core/src/scripts/slashs/citation.rs:152"),
                            interaction: BroadcastInteraction::default()
                                .set_name("citation")
                                .set_type(BroadcastInteractionType::SlashCommand),
                            details: BroadcastDetails::default()
                                .add("error", "Cannot get the citation's text")
                                .add("reason", "The text is not a string"),
                            ctx.skynet.as_ref()
                        );

                        return;
                    }
                },
                None => {
                    internal_error(ctx, &payload.interaction, local , "12004").await;

                    broadcast_error!(
                        localisation: BroadcastLocalisation::default()
                            .set_guild(payload.interaction.guild_id.clone())
                            .set_channel(payload.interaction.channel_id.clone())
                            .set_code_path("core/src/scripts/slashs/citation.rs:171"),
                        interaction: BroadcastInteraction::default()
                            .set_name("citation")
                            .set_type(BroadcastInteractionType::SlashCommand),
                        details: BroadcastDetails::default()
                            .add("error", "Cannot get the citation's text")
                            .add("reason", "The text is not a string"),
                        ctx.skynet.as_ref()
                    );
                    return;
                }
            }
        },
        None => {
            internal_error(ctx, &payload.interaction, local , "12003").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/slashs/citation.rs:190"),
                interaction: BroadcastInteraction::default()
                    .set_name("citation")
                    .set_type(BroadcastInteractionType::SlashCommand),
                details: BroadcastDetails::default()
                    .add("error", "Cannot get the citation's text")
                    .add("reason", "The text is not a string"),
                ctx.skynet.as_ref()
            );
            return;
        }
    };

    let (author_name, author_avatar): (String, Option<String>) = match &payload.interaction.member {
        Some(member) => {
            if let Some(user) = &member.user {
                (
                    user.global_name.clone().unwrap_or("Unknown".to_string()),
                    user.avatar_url(512, true, "png")
                )
            } else {
                ("Unknown".to_string(), None)
            }
        },
        None => ("Unknown".to_string(), None)
    };

    let msg = MessageBuilder::new()
        .add_embed(
            Embed::new()
                .set_color(Color::from_hex("#ffffff"))
                .set_author(
                    Author::new()
                        .set_icon_url(author_avatar)
                        .set_name(author_name)
                )
                .set_description(
                    format!(
                        "> 📄 ** ** {t}",
                        t = assets::profanity::censure({
                            if text.len() > 512 { text[0..512].to_string() }
                            else { text }
                        })
                    )
                )
                .set_timestamp(Utc::now())
        );

    let channel_id = guild_data.citation_channel.unwrap();
    let citation = ctx.skynet.send_message(
        &channel_id.into(),
        msg,
        None
    ).await;

    match citation {
        Ok(Ok(_)) => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(guild_data.lang, "features::citation::sent"))
                    .set_ephemeral(true)
            ).await;
        },
        Ok(Err(_)) => {
            internal_error(ctx, &payload.interaction, local , "12007").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/slashs/citation.rs:268"),
                interaction: BroadcastInteraction::default()
                    .set_name("citation")
                    .set_type(BroadcastInteractionType::SlashCommand),
                details: BroadcastDetails::default()
                    .add("error", "Cannot send the citation")
                    .add("reason", "An error occured while sending the citation"),
                ctx.skynet.as_ref()
            );
        }
        Err(e) => {
            error!(target: "Runtime", "An error occured while sending a citation: {e:#?}");
            internal_error(ctx, &payload.interaction, local , "12008").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/slashs/citation.rs:283"),
                interaction: BroadcastInteraction::default()
                    .set_name("citation")
                    .set_type(BroadcastInteractionType::SlashCommand),
                details: BroadcastDetails::default()
                    .add("error", format!("{e:#?}"))
                    .add("reason", "An error occured while sending the citation"),
                ctx.skynet.as_ref()
            );
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

async fn not_enabled(ctx: &Context, payload: &InteractionCreate) {
    let guild_locale = get_guild_locale(&payload.interaction.guild_locale);
    let _ = payload.interaction.reply(
        &ctx.skynet,
        MessageBuilder::new()
            .set_content(message!(guild_locale, "features::citation::not_enabled"))
            .set_ephemeral(true)
    ).await;
}

async fn no_valid_channel(ctx: &Context, payload: &InteractionCreate) {
    let guild_locale = get_guild_locale(&payload.interaction.guild_locale);
    let _ = payload.interaction.reply(
        &ctx.skynet,
        MessageBuilder::new()
            .set_content(message!(guild_locale, "features::citation::no_valid_channel"))
            .set_ephemeral(true)
    ).await;
}