use std::ops::Deref;
use log::error;
use client::manager::events::Context;
use client::models::components::message_components::Component;
use client::models::events::InteractionCreate;
use client::models::message::MessageBuilder;
use database::Database;
use translation::message;
use crate::crates::cookies;
use crate::scripts::{get_guild_locale, get_user_id};
use crate::scripts::slashs::internal_error;
use crate::crates::error_broadcaster::*;
use crate::broadcast_error;

const TOLERANCE: usize = 1;

pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
    let local = get_guild_locale(&payload.interaction.guild_locale);

    let db = match ctx.get_data::<Database>().await {
        Some(d) => d,
        None => {
            internal_error(ctx, &payload.interaction, local, "30001").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:29"),
                interaction: BroadcastInteraction::default()
                    .set_name("cookie_quiz_answer")
                    .set_type(BroadcastInteractionType::Modal),
                details: BroadcastDetails::default()
                    .add("code", "30001")
                    .add("error", "Cannot get the database from the Context"),
                ctx.skynet.as_ref()
            );

            return;
        }
    };

    let pool = db.get_pool().await;
    let requests = db.get_requests().await;

    let user_id = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
        Some(id) => id,
        None => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new().set_content(message!(local, "errors::cannot_get_user_id"))
            ).await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:56"),
                interaction: BroadcastInteraction::default()
                    .set_name("cookie_quiz_answer")
                    .set_type(BroadcastInteractionType::Modal),
                details: BroadcastDetails::default()
                    .add("code", "30002")
                    .add("error", "Cannot get the user id"),
                ctx.skynet.as_ref()
            );

            return;
        }
    };

    // get the user question
    let user_question = match cookies::quiz::get_user(&pool, &requests, &user_id).await {
        Ok(u) => match u {
            Some(q) => q,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new().set_content(message!(local, "features::cookies::quiz::answers::no_question_assigned"))
                ).await;
                return;
            }
        },
        Err(e) => {
            error!(target: "Runtime", "Cannot fetch the user question, therefor he responded to the modal: {e:#?}");
            internal_error(ctx, &payload.interaction, local, "30002").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:85"),
                interaction: BroadcastInteraction::default()
                    .set_name("cookie_quiz_answer")
                    .set_type(BroadcastInteractionType::Modal),
                details: BroadcastDetails::default()
                    .add("code", "30003")
                    .add("error", "Cannot fetch the user question"),
                ctx.skynet.as_ref()
            );

            return;
        }
    };

    if user_question.completed {
        // nop
        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(message!(local, "features::cookies::quiz::already_given"))
        ).await;
        return;
    }

    let all_possible_answers = match cookies::quiz::get_all_possible_answers(&pool, &requests, &user_question.id).await {
        Ok(a) => a,
        Err(e) => {
            error!(target: "Runtime", "Cannot fetch all possible answers from the user question, therefor he responded to the modal and he exist in the db: {e:#?}");
            internal_error(ctx, &payload.interaction, local, "30003").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:115"),
                interaction: BroadcastInteraction::default()
                    .set_name("cookie_quiz_answer")
                    .set_type(BroadcastInteractionType::Modal),
                details: BroadcastDetails::default()
                    .add("code", "30004")
                    .add("error", "Cannot fetch all possible answers from the user question"),
                ctx.skynet.as_ref()
            );

            return;
        }
    };

    let answer = match payload.interaction.data.as_ref() {
        Some(d) => match d.components.as_ref() {
            Some(components) => {

                let component = match components.get(0) {
                    Some(c) => c,
                    None => {
                        internal_error(ctx, &payload.interaction, local, "30006").await;

                        broadcast_error!(
                            localisation: BroadcastLocalisation::default()
                                .set_guild(payload.interaction.guild_id.clone())
                                .set_channel(payload.interaction.channel_id.clone())
                                .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:141"),
                            interaction: BroadcastInteraction::default()
                                .set_name("cookie_quiz_answer")
                                .set_type(BroadcastInteractionType::Modal),
                            details: BroadcastDetails::default()
                                .add("code", "30006")
                                .add("error", "Cannot get the first component"),
                            ctx.skynet.as_ref()
                        );

                        return;
                    }
                };

                let component = match component {
                    Component::ActionRow(r) => r,
                    _ => {
                        internal_error(ctx, &payload.interaction, local, "30006").await;

                        broadcast_error!(
                            localisation: BroadcastLocalisation::default()
                                .set_guild(payload.interaction.guild_id.clone())
                                .set_channel(payload.interaction.channel_id.clone())
                                .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:160"),
                            interaction: BroadcastInteraction::default()
                                .set_name("cookie_quiz_answer")
                                .set_type(BroadcastInteractionType::Modal),
                            details: BroadcastDetails::default()
                                .add("code", "30006")
                                .add("error", "Cannot get the first component"),
                            ctx.skynet.as_ref()
                        );

                        return;
                    }
                };

                let finder = component.components.iter()
                    .find(|c| {
                        if let Component::TextInput(text_input) = c {
                            text_input.custom_id == "COOKIE_USER_QUIZ_ANSWER_FIELD"
                        } else {
                            false
                        }
                    });
                match finder {
                    Some(f) => {
                        if let Component::TextInput(text_input) = f {
                            match text_input.value.clone() {
                                Some(v) => v.to_string(),
                                None => {
                                    internal_error(ctx, &payload.interaction, local, "30007").await;

                                    broadcast_error!(
                                        localisation: BroadcastLocalisation::default()
                                            .set_guild(payload.interaction.guild_id.clone())
                                            .set_channel(payload.interaction.channel_id.clone())
                                            .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:185"),
                                        interaction: BroadcastInteraction::default()
                                            .set_name("cookie_quiz_answer")
                                            .set_type(BroadcastInteractionType::Modal),
                                        details: BroadcastDetails::default()
                                            .add("code", "30007")
                                            .add("error", "Cannot get the value of the text input"),
                                        ctx.skynet.as_ref()
                                    );

                                    return;
                                }
                            }
                        } else {
                            internal_error(ctx, &payload.interaction, local, "30006").await;

                            broadcast_error!(
                                localisation: BroadcastLocalisation::default()
                                    .set_guild(payload.interaction.guild_id.clone())
                                    .set_channel(payload.interaction.channel_id.clone())
                                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:230"),
                                interaction: BroadcastInteraction::default()
                                    .set_name("cookie_quiz_answer")
                                    .set_type(BroadcastInteractionType::Modal),
                                details: BroadcastDetails::default()
                                    .add("code", "30006")
                                    .add("error", "Cannot get the first component"),
                                ctx.skynet.as_ref()
                            );

                            return;
                        }
                    },
                    None => {
                        internal_error(ctx, &payload.interaction, local, "30006").await;

                        broadcast_error!(
                            localisation: BroadcastLocalisation::default()
                                .set_guild(payload.interaction.guild_id.clone())
                                .set_channel(payload.interaction.channel_id.clone())
                                .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:250"),
                            interaction: BroadcastInteraction::default()
                                .set_name("cookie_quiz_answer")
                                .set_type(BroadcastInteractionType::Modal),
                            details: BroadcastDetails::default()
                                .add("code", "30006")
                                .add("error", "Cannot get the first component"),
                            ctx.skynet.as_ref()
                        );

                        return;
                    }
                }
            },
            None => {
                internal_error(ctx, &payload.interaction, local, "30005").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:267"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookie_quiz_answer")
                        .set_type(BroadcastInteractionType::Modal),
                    details: BroadcastDetails::default()
                        .add("code", "30005")
                        .add("error", "Cannot get the interaction data"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        },
        None => {
            internal_error(ctx, &payload.interaction, local, "30004").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:291"),
                interaction: BroadcastInteraction::default()
                    .set_name("cookie_quiz_answer")
                    .set_type(BroadcastInteractionType::Modal),
                details: BroadcastDetails::default()
                    .add("code", "30005")
                    .add("error", "Cannot get the interaction data"),
                ctx.skynet.as_ref()
            );

            return;
        }
    };

    let is_valid = cookies::quiz::check_answer_validity(
        answer.to_lowercase().as_str(),
        &all_possible_answers,
        TOLERANCE
    );

    if is_valid {
        // declare the question as completed :)
        let question_completed_result = cookies::quiz::question_completed(
            &pool,
            &requests,
            &user_id,
        ).await;

        if let Err(e) = question_completed_result {
            error!(target: "Runtime", "Cannot declare the user cookie question as completed: {e:#?}");
            internal_error(ctx, &payload.interaction, local, "30008").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:327"),
                interaction: BroadcastInteraction::default()
                    .set_name("cookie_quiz_answer")
                    .set_type(BroadcastInteractionType::Modal),
                details: BroadcastDetails::default()
                    .add("code", "30008")
                    .add("error", "Cannot declare the user cookie question as completed"),
                ctx.skynet.as_ref()
            );

            return;
        }

        // give the cookie
        let cookie_given = sqlx::query(requests.users.cookies.give_cookie_from_system.as_str())
            .bind(&user_id.to_string())
            .execute(pool.deref())
            .await;

        if let Err(e) = cookie_given {
            error!(target: "Runtime", "Cannot give a cookie to the user: {e:#?}");
            internal_error(ctx, &payload.interaction, local, "30009").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:350"),
                interaction: BroadcastInteraction::default()
                    .set_name("cookie_quiz_answer")
                    .set_type(BroadcastInteractionType::Modal),
                details: BroadcastDetails::default()
                    .add("code", "30009")
                    .add("error", "Cannot give a cookie to the user"),
                ctx.skynet.as_ref()
            );

            return;
        }

        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(
                    message!(local, "features::cookies::quiz::answers::good")
                )
        ).await;
    } else {
        // naaa, give him 3 nuggets
        let question_completed_result = cookies::quiz::question_completed(
            &pool,
            &requests,
            &user_id,
        ).await;

        if let Err(e) = question_completed_result {
            error!(target: "Runtime", "Cannot declare the user cookie question as completed: {e:#?}");
            internal_error(ctx, &payload.interaction, local, "30008").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:386"),
                interaction: BroadcastInteraction::default()
                    .set_name("cookie_quiz_answer")
                    .set_type(BroadcastInteractionType::Modal),
                details: BroadcastDetails::default()
                    .add("code", "30008")
                    .add("error", "Cannot declare the user cookie question as completed"),
                ctx.skynet.as_ref()
            );

            return;
        }

        // give the nuggets
        let nuggets_given = sqlx::query(requests.users.cookies.increase_nuggets.as_str())
            .bind(3)
            .bind(&user_id.to_string())
            .execute(pool.deref())
            .await;

        if let Err(e) = nuggets_given {
            error!(target: "Runtime", "Cannot give 3 nuggets to the user: {e:#?}");
            internal_error(ctx, &payload.interaction, local, "30010").await;

            broadcast_error!(
                localisation: BroadcastLocalisation::default()
                    .set_guild(payload.interaction.guild_id.clone())
                    .set_channel(payload.interaction.channel_id.clone())
                    .set_code_path("core/src/scripts/modal/cookie_quiz_answer.rs:414"),
                interaction: BroadcastInteraction::default()
                    .set_name("cookie_quiz_answer")
                    .set_type(BroadcastInteractionType::Modal),
                details: BroadcastDetails::default()
                    .add("code", "30010")
                    .add("error", "Cannot give 3 nuggets to the user"),
                ctx.skynet.as_ref()
            );

            return;
        }

        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(
                    message!(local, "features::cookies::quiz::answers::no_this_time")
                )
        ).await;
    }

}