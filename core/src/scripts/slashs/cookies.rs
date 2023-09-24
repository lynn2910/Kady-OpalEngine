//! Error code: 15xxx

use client::manager::events::Context;
use client::models::events::InteractionCreate;
use client::models::message::MessageBuilder;
use database::Database;
use database::model::users::User;
use translation::message;
use crate::scripts::{get_guild_locale, get_user_id};
use crate::scripts::slashs::internal_error;
use crate::crates::error_broadcaster::*;
use crate::broadcast_error;

mod give_cookies {
    use std::ops::Deref;
    use log::error;
    use sqlx::MySqlPool;
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;
    use client::models::interaction::{InteractionDataOption, InteractionDataOptionValue};
    use client::models::message::MessageBuilder;
    use client::models::user::UserId;
    use database::dynamic_requests::DynamicRequest;
    use database::model::users::UserCookie;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::crates::cookies::notify_new_cookie;
    use crate::scripts::{get_user, get_user_id};
    use crate::scripts::slashs::{internal_error, internal_error_deferred};
    use crate::crates::error_broadcaster::*;
    use crate::broadcast_error;

    pub(crate) async fn triggered(
        ctx: &Context,
        payload: &InteractionCreate,
        local: String,
        subcommand: &InteractionDataOption,
        pool: &MySqlPool,
        requests: &DynamicRequest
    )
    {
        let options = match &subcommand.options {
            Some(opts) => opts,
            _ => {
                internal_error(ctx, &payload.interaction, local, "15003").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/cookies.rs:give_cookies::triggered:51"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookies/give")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("code", "15003")
                        .add("reason", "Cannot acquire the interaction data options"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        // find user ID
        let user_id = {
            let opt = options.iter().find(|o| o.name == "user");

            if let Some(option) = opt {
                match &option.value {
                    Some(InteractionDataOptionValue::String(v)) => v.to_string(),
                    _ => {
                        internal_error(ctx, &payload.interaction, local, "15004").await;

                        broadcast_error!(
                            localisation: BroadcastLocalisation::default()
                                .set_guild(payload.interaction.guild_id.clone())
                                .set_channel(payload.interaction.channel_id.clone())
                                .set_code_path("core/src/scripts/slashs/cookies.rs:give_cookies::triggered:79"),
                            interaction: BroadcastInteraction::default()
                                .set_name("cookies/give")
                                .set_type(BroadcastInteractionType::SlashCommand),
                            details: BroadcastDetails::default()
                                .add("code", "15004")
                                .add("reason", "Cannot acquire the user ID"),
                            ctx.skynet.as_ref()
                        );

                        return;
                    }
                }
            } else {
                internal_error(ctx, &payload.interaction, local, "15004").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/cookies.rs:give_cookies::triggered:99"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookies/give")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("code", "15004")
                        .add("reason", "Cannot acquire the user ID"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        let author_id = get_user_id(&payload.interaction.user, &payload.interaction.member).unwrap_or(UserId::from(""));

        // check if user is a bot
        {
            let user = get_user(ctx, &UserId::from(&user_id)).await;
            match user {
                Some(u) if u.bot.unwrap_or(false) => {
                    let _ = payload.interaction.reply(
                        &ctx.skynet,
                        MessageBuilder::new()
                            .set_content(message!(local, "errors::not_for_bot"))
                            .set_ephemeral(true)
                    ).await;
                    return;
                },
                Some(u) if u.id.to_string() == author_id.to_string() => {
                    let _ = payload.interaction.reply(
                        &ctx.skynet,
                        MessageBuilder::new()
                            .set_content(message!(local, "features::cookies::not_yourself"))
                            .set_ephemeral(true)
                    ).await;
                    return;
                }
                Some(_) => {}
                _ => {
                    let _ = payload.interaction.reply(
                        &ctx.skynet,
                        MessageBuilder::new()
                            .set_content(message!(local, "errors::cannot_acquire_user"))
                            .set_ephemeral(true)
                    ).await;

                    broadcast_error!(
                        localisation: BroadcastLocalisation::default()
                            .set_guild(payload.interaction.guild_id.clone())
                            .set_channel(payload.interaction.channel_id.clone())
                            .set_code_path("core/src/scripts/slashs/cookies.rs:give_cookies::triggered:150"),
                        interaction: BroadcastInteraction::default()
                            .set_name("cookies/give")
                            .set_type(BroadcastInteractionType::SlashCommand),
                        details: BroadcastDetails::default()
                            .add("code", "15004")
                            .add("reason", "Cannot acquire the user"),
                        ctx.skynet.as_ref()
                    );

                    return;
                }
            }
        }

        let cookies_given = {
            let opt = options.iter().find(|o| o.name == "number");

            if let Some(option) = opt {
                match &option.value {
                    Some(InteractionDataOptionValue::Integer(v)) if v > &0i64 => *v,
                    Some(InteractionDataOptionValue::Integer(_)) => {
                        let _ = payload.interaction.reply(
                            &ctx.skynet,
                            MessageBuilder::new()
                                .set_content(message!(local, "features::cookies::cookies_number_null"))
                                .set_ephemeral(true)
                        ).await;
                        return;
                    }
                    Some(InteractionDataOptionValue::Double(double)) => {
                        if &double.round() != double {
                            let _ = payload.interaction.reply(
                                &ctx.skynet,
                                MessageBuilder::new()
                                    .set_content(message!(local, "features::cookies::cookies_number_as_float"))
                                    .set_ephemeral(true)
                            ).await;
                            return;
                        } else {
                            double.round() as i64
                        }
                    }
                    _ => {
                        internal_error(ctx, &payload.interaction, local, "15005").await;

                        broadcast_error!(
                            localisation: BroadcastLocalisation::default()
                                .set_guild(payload.interaction.guild_id.clone())
                                .set_channel(payload.interaction.channel_id.clone())
                                .set_code_path("core/src/scripts/slashs/cookies.rs:give_cookies::triggered:200"),
                            interaction: BroadcastInteraction::default()
                                .set_name("cookies/give")
                                .set_type(BroadcastInteractionType::SlashCommand),
                            details: BroadcastDetails::default()
                                .add("code", "15005")
                                .add("reason", "Cannot acquire the number of cookies to give"),
                            ctx.skynet.as_ref()
                        );

                        return;
                    }
                }
            } else {
                internal_error(ctx, &payload.interaction, local, "15005").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/cookies.rs:give_cookies::triggered:220"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookies/give")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("code", "15005")
                        .add("reason", "Cannot acquire the number of cookies to give"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        // defer the interaction
        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        // ensure the existence of the User in the database
        {
            let _ = database::model::users::User::ensure(pool, requests.users.ensure.as_str(), user_id.as_str()).await;
        }

        let mut author_cookies = match UserCookie::get_all_cookies(pool, &requests.users.cookies.get, &author_id).await {
            Ok(cookies) => cookies,
            Err(e) => {
                error!(target: "Runtime", "An error occured while fetching all cookies to donate: {e:#?}");
                internal_error_deferred(ctx, &payload.interaction, local, "15006").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/cookies.rs:give_cookies::triggered:252"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookies/give")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("code", "15006")
                        .add("reason", "Cannot fetch all cookies to donate"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        // check if the author has enough cookies :)
        if author_cookies.len() < cookies_given as usize {
            let _ = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(local, "features::cookies::not_enough_cookies"))
            ).await;
            return;
        }

        // sort the cookies to have the older in first
        author_cookies.sort_by(|a, b| a.timestamp.timestamp().cmp(&b.timestamp.timestamp()));

        let guild = payload.interaction.guild_id.clone().map(|id| id.to_string()).unwrap_or("NULL".into());
        for _ in 0..cookies_given {
            // remove one cookie from the author
            let last_cookie = author_cookies.pop();
            if last_cookie.is_none() {
                internal_error_deferred(ctx, &payload.interaction, local, "15007").await;
                return
            }
            let last_cookie = last_cookie.unwrap();

            let removed_cookie = sqlx::query(requests.users.cookies.remove_cookie.as_str())
                .bind(&author_id.to_string())
                .execute(pool.deref())
                .await;

            if let Err(e) = removed_cookie {
                error!(target: "Runtime", "An error occured while removing a cookies that was donate: {e:#?}");
                internal_error_deferred(ctx, &payload.interaction, local, "15008").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/cookies.rs:give_cookies::triggered:302"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookies/give")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("code", "15008")
                        .add("reason", "Cannot remove a cookie that was donated"),
                    ctx.skynet.as_ref()
                );

                return;
            }

            // add a new cookie :)))
            let cookie_given = sqlx::query(requests.users.cookies.give_cookie_in_guild.as_str())
                .bind(last_cookie.user_from)
                .bind(&user_id)
                .bind(&guild)
                .execute(pool.deref())
                .await;

            if let Err(e) = cookie_given {
                error!(target: "Runtime", "An error occured while removing a cookies that was donate: {e:#?}");
                internal_error_deferred(ctx, &payload.interaction, local, "15009").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/cookies.rs:give_cookies::triggered:331"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookies/give")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("code", "15009")
                        .add("reason", "Cannot give a cookie"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        }

        // confirm cookies givens
        let _ = payload.interaction.update(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(
                    message!(
                        local.clone(),
                        "features::cookies::cookies_given",
                        Formatter::new()
                            .add("user", user_id.as_str())
                            .add("cookies", cookies_given)
                    )
                )
        ).await;

        let author_name = {
            if let Some(user) = &payload.interaction.user {
                user.global_name.clone().unwrap_or(user.username.clone())
            } else {
                match &payload.interaction.member {
                    Some(m) => {
                        match &m.user {
                            Some(user) => user.global_name.clone().unwrap_or(user.username.clone()),
                            None => "__internal_core_error_manager__".into()
                        }
                    },
                    None => "__internal_core_error_manager__".into()
                }
            }
        };

        let cookies_count = UserCookie::get_cookies_count(pool, requests.users.cookies.get.as_str(), user_id.as_str()).await;

        notify_new_cookie(
            &ctx.skynet,
            author_name,
            user_id.into(),
            cookies_given as u64,
            cookies_count.unwrap_or(0) as u64
        ).await;
    }
}

mod daily {
    use chrono::Utc;
    use log::error;
    use serde_json::Value;
    use sqlx::MySqlPool;
    use client::manager::events::Context;
    use client::models::components::Emoji;
    use client::models::components::message_components::{ActionRow, Button, ButtonStyle, Component};
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use database::dynamic_requests::DynamicRequest;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::crates::cookies;
    use crate::scripts::get_user_id;
    use crate::scripts::slashs::internal_error_deferred;
    use crate::crates::error_broadcaster::*;
    use crate::broadcast_error;

    pub(crate) async fn triggered(
        ctx: &Context,
        payload: &InteractionCreate,
        local: String,
        pool: &MySqlPool,
        requests: &DynamicRequest
    )
    {
        // a check if the author is registered in the database had been accomplished before, and therefore we can
        // immediately fetch the user informations about the quiz :D

        let user_id = get_user_id(&payload.interaction.user, &payload.interaction.member);
        if user_id.is_none() {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(local, "errors::cannot_get_user_id"))
                    .set_ephemeral(true)
            ).await;

            broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/cookies.rs:daily:291"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookies/daily")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("reason", "Cannot acquire user ID"),
                    ctx.skynet.as_ref()
                );

            return;
        }
        let user_id  = user_id.unwrap();

        // defer the interaction
        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let user_quiz_query = match cookies::quiz::get_user(pool, requests, &user_id).await {
            Ok(u) => u,
            Err(e) => {
                error!(target: "Runtime", "Cannot fetch the user informations for the daily's cookie quiz: {e:#?}");
                internal_error_deferred(ctx, &payload.interaction, local, "15010").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/cookies.rs:daily:313"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookies/daily")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("code", "15010")
                        .add("reason", "Cannot fetch the user informations for the daily's cookie quiz"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        if let Some(old_quiz) = user_quiz_query {
            // he have a old quiz saved (or the question is for this day)

            if old_quiz.completed {
                // nop
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(
                            local,
                            "features::cookies::quiz::already_given",
                            // the date at which the user can retry the quiz is not stored in the database
                            // but the time at which the user can get a new question is at 00:00:00 UTC
                            // We need to give a UNIX timestamp to the user
                            Formatter::new().add("retry_in", {
                                let now = Utc::now();
                                let tomorrow = now.date_naive().succ_opt().unwrap_or(Default::default());
                                let tomorrow = tomorrow.and_hms_opt(0, 0, 0).unwrap_or(Default::default());
                                tomorrow.timestamp()
                            })
                        ))
                ).await;
                return;
            }

            let questions_traductions: Value = message!(&local, "features::cookies::quiz::questions").into();
            let this_question_traduction = {
                if let Some(object) = questions_traductions.as_object() {
                    match object.get(&old_quiz.id) {
                        Some(q) => q.clone(),
                        None => {
                            internal_error_deferred(ctx, &payload.interaction, local, "15013").await;

                            broadcast_error!(
                                localisation: BroadcastLocalisation::default()
                                    .set_guild(payload.interaction.guild_id.clone())
                                    .set_channel(payload.interaction.channel_id.clone())
                                    .set_code_path("core/src/scripts/slashs/cookies.rs:daily:369"),
                                interaction: BroadcastInteraction::default()
                                    .set_name("cookies/daily")
                                    .set_type(BroadcastInteractionType::SlashCommand),
                                details: BroadcastDetails::default()
                                    .add("code", "15013")
                                    .add("reason", "Cannot fetch the question traduction"),
                                ctx.skynet.as_ref()
                            );

                            return;
                        }
                    }
                } else {
                    internal_error_deferred(ctx, &payload.interaction, local, "15012").await;

                    broadcast_error!(
                        localisation: BroadcastLocalisation::default()
                            .set_guild(payload.interaction.guild_id.clone())
                            .set_channel(payload.interaction.channel_id.clone())
                            .set_code_path("core/src/scripts/slashs/cookies.rs:daily:389"),
                        interaction: BroadcastInteraction::default()
                            .set_name("cookies/daily")
                            .set_type(BroadcastInteractionType::SlashCommand),
                        details: BroadcastDetails::default()
                            .add("code", "15012")
                            .add("reason", "Cannot fetch the questions traductions"),
                        ctx.skynet.as_ref()
                    );

                    return;
                }
            };

            // finally, give it to the user !
            let _ = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            &local,
                            "features::cookies::quiz::give_question",
                            Formatter::new().add("question", this_question_traduction.as_str().unwrap_or("__core::translation::ERROR_FORMATTING_QUESTION"))
                        )
                    )
                    .add_component(
                        Component::ActionRow(
                            ActionRow::new()
                                .add_component(Component::Button(
                                    Button::new("ANSWER_COOKIES_QUIZ")
                                        .set_label(message!(&local, "features::cookies::quiz::answer_button"))
                                        .set_style(ButtonStyle::Secondary)
                                        .set_emoji(Emoji::new(None, "🤔"))
                                        .set_disabled(false)
                                ))
                        )
                    )
            ).await;

        } else {
            // he isn't registered, so we can give him immediately the question

            let question = match cookies::quiz::get_random_question(pool, requests).await {
                Ok(q) => q,
                Err(e) => {
                    error!(target: "Runtime", "An error occured while acquiring a random question for the daily cookie quiz: {e:#?}");
                    internal_error_deferred(ctx, &payload.interaction, local, "15011").await;

                    broadcast_error!(
                        localisation: BroadcastLocalisation::default()
                            .set_guild(payload.interaction.guild_id.clone())
                            .set_channel(payload.interaction.channel_id.clone())
                            .set_code_path("core/src/scripts/slashs/cookies.rs:daily:441"),
                        interaction: BroadcastInteraction::default()
                            .set_name("cookies/daily")
                            .set_type(BroadcastInteractionType::SlashCommand),
                        details: BroadcastDetails::default()
                            .add("code", "15011")
                            .add("reason", "Cannot fetch a random question"),
                        ctx.skynet.as_ref()
                    );

                    return;
                }
            };

            let questions_traductions: Value = message!(&local, "features::cookies::quiz::questions").into();
            let this_question_traduction = {
                if let Some(object) = questions_traductions.as_object() {
                    match object.get(&question.id) {
                        Some(q) => q.clone(),
                        None => {
                            internal_error_deferred(ctx, &payload.interaction, local, "15013").await;

                            broadcast_error!(
                                localisation: BroadcastLocalisation::default()
                                    .set_guild(payload.interaction.guild_id.clone())
                                    .set_channel(payload.interaction.channel_id.clone())
                                    .set_code_path("core/src/scripts/slashs/cookies.rs:daily:467"),
                                interaction: BroadcastInteraction::default()
                                    .set_name("cookies/daily")
                                    .set_type(BroadcastInteractionType::SlashCommand),
                                details: BroadcastDetails::default()
                                    .add("code", "15013")
                                    .add("reason", "Cannot fetch the question traduction"),
                                ctx.skynet.as_ref()
                            );

                            return;
                        }
                    }
                } else {
                    internal_error_deferred(ctx, &payload.interaction, local, "15012").await;

                    broadcast_error!(
                        localisation: BroadcastLocalisation::default()
                            .set_guild(payload.interaction.guild_id.clone())
                            .set_channel(payload.interaction.channel_id.clone())
                            .set_code_path("core/src/scripts/slashs/cookies.rs:daily:487"),
                        interaction: BroadcastInteraction::default()
                            .set_name("cookies/daily")
                            .set_type(BroadcastInteractionType::SlashCommand),
                        details: BroadcastDetails::default()
                            .add("code", "15012")
                            .add("reason", "Cannot fetch the questions traductions"),
                        ctx.skynet.as_ref()
                    );

                    return;
                }
            };

            // register the question :)
            let insert_result = cookies::quiz::insert_user(pool, requests, &user_id, &question.id).await;
            if let Err(e) = insert_result {
                error!(target: "Runtime", "An error occured while inserting the user in the quiz table: {e:#?}");
                internal_error_deferred(ctx, &payload.interaction, local, "15014").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/cookies.rs:daily:511"),
                    interaction: BroadcastInteraction::default()
                        .set_name("cookies/daily")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("code", "15014")
                        .add("reason", "Cannot insert the user in the quiz table"),
                    ctx.skynet.as_ref()
                );

                return;
            }

            // finally, give it to the user !
            let _ = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            &local,
                            "features::cookies::quiz::give_question",
                            Formatter::new().add("question", this_question_traduction.as_str().unwrap_or("__core::translation::ERROR_FORMATTING_QUESTION"))
                        )
                    )
                    .add_component(
                        Component::ActionRow(
                            ActionRow::new()
                                .add_component(Component::Button(
                                    Button::new("ANSWER_COOKIES_QUIZ")
                                        .set_label(message!(&local, "features::cookies::quiz::answer_button"))
                                        .set_style(ButtonStyle::Secondary)
                                        .set_emoji(Emoji::new(None, "🤔"))
                                        .set_disabled(false)
                                ))
                        )
                    )
            ).await;
        }
    }
}

const AVAILABLE_CATEGORIES: [&str; 2] = ["daily", "donate"];

pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
    let local = get_guild_locale(&payload.interaction.guild_locale);

    let data = if let Some(d) = &payload.interaction.data {
        d
    } else {
        internal_error(ctx, &payload.interaction, local, "15001").await;

        broadcast_error!(
            localisation: BroadcastLocalisation::default()
                .set_guild(payload.interaction.guild_id.clone())
                .set_channel(payload.interaction.channel_id.clone())
                .set_code_path("core/src/scripts/slashs/cookies.rs:triggered:705"),
            interaction: BroadcastInteraction::default()
                .set_name("cookies")
                .set_type(BroadcastInteractionType::SlashCommand),
            details: BroadcastDetails::default()
                .add("code", "15001")
                .add("reason", "Cannot acquire the interaction data"),
            ctx.skynet.as_ref()
        );

        return;
    };

    if data.options.is_none() {
        internal_error(ctx, &payload.interaction, local, "15002").await;

        broadcast_error!(
            localisation: BroadcastLocalisation::default()
                .set_guild(payload.interaction.guild_id.clone())
                .set_channel(payload.interaction.channel_id.clone())
                .set_code_path("core/src/scripts/slashs/cookies.rs:triggered:725"),
            interaction: BroadcastInteraction::default()
                .set_name("cookies")
                .set_type(BroadcastInteractionType::SlashCommand),
            details: BroadcastDetails::default()
                .add("code", "15002")
                .add("reason", "Cannot acquire the interaction data options"),
            ctx.skynet.as_ref()
        );

        return;
    }
    let options = data.options.as_ref().unwrap();

    let subcommand = options.iter().find(|opt| AVAILABLE_CATEGORIES.contains(&opt.name.as_str()));

    match subcommand {
        Some(sub) => {
            match sub.name.as_str() {
                "donate" => {
                    let db = ctx.get_data::<Database>().await.expect("Cannot acquire the Database");

                    let pool = db.get_pool().await;
                    let requests = db.get_requests().await;

                    {
                        let author_id = get_user_id(&payload.interaction.user, &payload.interaction.member);

                        if let Some(author_id) = author_id {
                            let _ = User::ensure(&pool, requests.users.ensure.as_str(), author_id).await;
                        }
                    }

                    give_cookies::triggered(
                        ctx,
                        payload,
                        local,
                        sub,
                        &pool,
                        &requests
                    ).await;
                },
                "daily" => {
                    let db = ctx.get_data::<Database>().await.expect("Cannot acquire the Database");

                    let pool = db.get_pool().await;
                    let requests = db.get_requests().await;

                    {
                        let author_id = get_user_id(&payload.interaction.user, &payload.interaction.member);

                        if let Some(author_id) = author_id {
                            let _ = User::ensure(&pool, requests.users.ensure.as_str(), author_id).await;
                        }
                    }

                    daily::triggered(
                        ctx,
                        payload,
                        local,
                        &pool,
                        &requests
                    ).await;
                }
                _ => {
                    let _ = payload.interaction.reply(
                        &ctx.skynet,
                        MessageBuilder::new()
                            .set_content(message!(local, "features::cookies::invalid_category"))
                    ).await;
                }
            }
        }
        _ => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(local, "features::cookies::no_category_provided"))
            ).await;
        }
    }
}