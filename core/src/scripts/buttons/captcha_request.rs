//! Unique error ID for this file: 10xxx

use log::error;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use client::manager::events::Context;
use client::models::components::message_components::{ActionRow, Button, ButtonStyle, Component};
use client::models::events::InteractionCreate;
use client::models::message::{AttachmentBuilder, message_flags, MessageAttachmentBuilder, MessageBuilder, MessageFlags};
use database::{Database, model};
use features::captcha;
use translation::fmt::formatter::Formatter;
use translation::message;
use crate::crates::error_broadcaster::*;
use crate::broadcast_error;
use crate::scripts::{get_guild_locale, QueryParams};

pub(in crate::scripts) async fn triggered(ctx: &Context, payload: &InteractionCreate, _query: QueryParams) {
    let guild_id = match &payload.interaction.guild_id {
        Some(id) => id.clone(),
        _ => return
    };

    let user_id = match &payload.interaction.member {
        Some(member) => match &member.user {
            Some(user) => user.id.clone(),
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(
                            message!(
                                get_guild_locale(&payload.interaction.guild_locale),
                                "errors::cannot_get_user_id"
                            )
                        )
                        .set_ephemeral(true)
                ).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(Some(guild_id))
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_user(payload.interaction.user.clone())
                        .set_code_path("core::scripts::button::captcha_request.rs:45"),
                    interaction: BroadcastInteraction::default()
                        .set_type(BroadcastInteractionType::Button)
                        .set_name("captcha_request"),
                    details: BroadcastDetails::default()
                        .add("reason", "Cannot get user id after un-packaging &User from &GuildMember"),
                    ctx.skynet.as_ref()
                );

                return
            }
        },
        None => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            get_guild_locale(&payload.interaction.guild_locale),
                            "errors::cannot_get_user_id"
                        )
                    )
                    .set_ephemeral(true)
            ).await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(Some(guild_id))
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_user(payload.interaction.user.clone())
                    .set_code_path("core::scripts::button::captcha_request.rs:75"),
                interaction: BroadcastInteraction::default()
                    .set_type(BroadcastInteractionType::Button)
                    .set_name("captcha_request"),
                details: BroadcastDetails::default()
                    .add("reason", "Cannot get user id from the payload"),
                ctx.skynet.as_ref()
            );

            return
        }
    };


    // Check if the user has already requested a captcha
    // if yes, we will delete the old instance :/
    {
        let mut data = ctx.data.write().await;
        let captcha_container = match data.get_mut::<captcha::CaptchaContainer>() {
            Some(cc) => cc,
            _ => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(
                            message!(
                                get_guild_locale(&payload.interaction.guild_locale),
                                "errors::internal_error",
                                Formatter::new().add("code", "10000")
                            )
                        )
                        .set_ephemeral(true)
                ).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(Some(guild_id))
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_user(payload.interaction.user.clone())
                        .set_code_path("core::scripts::button::captcha_request.rs:114"),
                    interaction: BroadcastInteraction::default()
                        .set_type(BroadcastInteractionType::Button)
                        .set_name("captcha_request"),
                    details: BroadcastDetails::default()
                        .add("reason", "the structure features::captcha::CaptchaContainer is not present in the context data"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        if let Some(id) = captcha_container.get_instance_from_user(user_id.to_string()).await {
            captcha_container.remove_instance(id).await;
        }
    }



    let database = ctx.get_data::<Database>().await.expect("No database found");

    let guild_data = {
        let pool = database.get_pool().await;
        match model::guild::Guild::from_pool(&pool, database.get_requests().await.guilds.get.as_str(), &guild_id).await {
            Ok(guild_data) => guild_data,
            Err(e) => {
                error!("Error while fetching guild data: {:?}", e);
                cannot_get_guild_data(ctx, payload).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(Some(guild_id))
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_user(payload.interaction.user.clone())
                        .set_code_path("core::scripts::button::captcha_request.rs:149"),
                    interaction: BroadcastInteraction::default()
                        .set_type(BroadcastInteractionType::Button)
                        .set_name("captcha_request"),
                    details: BroadcastDetails::default()
                        .add("reason", "Cannot get guild data from the database")
                        .add("error", format!("{e:#?}")),
                    ctx.skynet.as_ref()
                );

                return;
            }
        }
    };

    if !guild_data.captcha_enabled.unwrap_or(false) {
        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(
                    message!(guild_data.lang, "features::captcha::disabled")
                )
                .set_ephemeral(true)
        ).await;
        return;
    }

    let difficulty = match guild_data.captcha_level {
        Some(1) => captcha::generator::Difficulty::Easy,
        Some(2) => captcha::generator::Difficulty::Medium,
        Some(3) => captcha::generator::Difficulty::Hard,
        _ => {
            cannot_get_level(ctx, payload).await;
            return;
        }
    };

    let model = match guild_data.captcha_model.unwrap_or(String::new()).as_str() {
        "amelia" => captcha::generator::CaptchaName::Amelia,
        "lucy" => captcha::generator::CaptchaName::Lucy,
        "mila" => captcha::generator::CaptchaName::Mila,
        _ => {
            cannot_get_model(ctx, payload).await;
            return;
        }
    };

    // send the loading status
    {
        let mut flags = MessageFlags::new();
        flags.add_flag(message_flags::EPHEMERAL);

        let msg = payload.interaction.defer(&ctx.skynet, Some(flags)).await;

        match msg {
            Err(e) => {
                error!(target: "Runtime", "An error occured in a captcha response: {e:#?}");

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(Some(guild_id))
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_user(payload.interaction.user.clone())
                        .set_code_path("core::scripts::button::captcha_request.rs:212"),
                    interaction: BroadcastInteraction::default()
                        .set_type(BroadcastInteractionType::Button)
                        .set_name("captcha_request"),
                    details: BroadcastDetails::default()
                        .add("reason", "Cannot defer the interaction")
                        .add("error", format!("{e:#?}")),
                    ctx.skynet.as_ref()
                );

                return;
            },
            Ok(Err(e)) => {
                error!(target: "Runtime", "An error occured in a captcha response: {e:#?}");
                return;
            },
            _ => {}
        }
    }

    let sample = captcha::generator::by_name(difficulty, model);

    let file = match sample.as_png() {
        Some(s) => AttachmentBuilder {
            bytes: s,
            content_type: "image/png".into(),
            description: None,
            filename: "captcha.png".into(),
            id: 0
        },
        _ => {
            cannot_generate(ctx, payload).await;
            return;
        }
    };


    let good_code = sample.chars();

    // Push the instance into the system & retrieve the uuid
    let instance_code = {
        let mut data = ctx.data.write().await;
        let captcha_container = match data.get_mut::<captcha::CaptchaContainer>() {
            Some(cc) => cc,
            _ => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(
                            message!(
                                guild_data.lang,
                                "errors::internal_error",
                                Formatter::new().add("code", "10001")
                            )
                        )
                        .set_ephemeral(true)
                ).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(Some(guild_id))
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_user(payload.interaction.user.clone())
                        .set_code_path("core::scripts::button::captcha_request.rs:275"),
                    interaction: BroadcastInteraction::default()
                        .set_type(BroadcastInteractionType::Button)
                        .set_name("captcha_request"),
                    details: BroadcastDetails::default()
                        .add("reason", "the structure features::captcha::CaptchaContainer is not present in the context data"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        captcha_container.push_new_instance(good_code.clone(), user_id.to_string()).await
    };
    let good_code: String = good_code.iter().collect();


    // shuffle the list of codes
    let codes = {
        let mut code_chunks = captcha::generate_random_code_chunk(captcha::chunk_nb_from_difficulty(difficulty), good_code.len());
        code_chunks.push(good_code.clone());

        let mut rng = thread_rng();
        code_chunks.shuffle(&mut rng);

        code_chunks
    };

    // build the message
    let mut msg = MessageBuilder::new()
        .set_content(message!(get_guild_locale(&payload.interaction.guild_locale), "features::captcha::request"))
        .add_attachment(MessageAttachmentBuilder {
            name: "captcha.png".into(),
            description: None,
            content_type: "image/png".into(),
            id: 0
        });

    // add buttons !
    {
        let mut action_row = ActionRow::new();

        for code in codes.iter() {
            action_row = action_row.add_component(
                Component::Button(
                    Button::new(format!("CAPTCHA_TRY&code={code}&instance={}", instance_code))
                        .set_label(code)
                        .set_style(ButtonStyle::Secondary)
                )
            );
        }

        msg = msg.add_component(Component::ActionRow(action_row));
    };

    // send message
    let msg_request = payload.interaction.update_with_files(
        &ctx.skynet,
        msg,
        vec![file]
    ).await;

    match msg_request {
        Ok(msg) => {
            if let Err(e) = msg {
                error!(target: "RuntimeError", "Code: 10002; Error: {e:#?}");
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(
                            message!(
                        guild_data.lang,
                        "errors::internal_error",
                        Formatter::new().add("code", "10002")
                    )
                        )
                ).await;
            }
        },
        Err(e) => {
            error!(target: "RuntimeError", "Code: 10003; Error: {e:#?}");
            let _ = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            guild_data.lang,
                            "errors::internal_error",
                            Formatter::new().add("code", "10003")
                        )
                    )
                    .set_ephemeral(true)
            ).await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(Some(guild_id))
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_user(payload.interaction.user.clone())
                    .set_code_path("core::scripts::button::captcha_request.rs:357"),
                interaction: BroadcastInteraction::default()
                    .set_type(BroadcastInteractionType::Button)
                    .set_name("captcha_request"),
                details: BroadcastDetails::default()
                    .add("reason", "Cannot send the captcha message")
                    .add("error", format!("{e:#?}")),
                ctx.skynet.as_ref()
            );
        }
    }
}




async fn cannot_get_level(ctx: &Context, payload: &InteractionCreate) {
    let guild_locale = get_guild_locale(&payload.interaction.guild_locale);
    let _ = payload.interaction.reply(
        &ctx.skynet,
        MessageBuilder::new()
            .set_content(message!(guild_locale, "errors::cannot_get_captcha_level"))
            .set_ephemeral(true)
    ).await;
}

async fn cannot_get_model(ctx: &Context, payload: &InteractionCreate) {
    let guild_locale = get_guild_locale(&payload.interaction.guild_locale);
    let _ = payload.interaction.reply(
        &ctx.skynet,
        MessageBuilder::new()
            .set_content(message!(guild_locale, "errors::cannot_get_captcha_model"))
            .set_ephemeral(true)
    ).await;
}

async fn cannot_generate(ctx: &Context, payload: &InteractionCreate) {
    let guild_locale = get_guild_locale(&payload.interaction.guild_locale);
    let _ = payload.interaction.update(
        &ctx.skynet,
        MessageBuilder::new()
            .set_content(message!(guild_locale, "errors::cannot_generate_captcha"))
            .set_ephemeral(true)
    ).await;
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