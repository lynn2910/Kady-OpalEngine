use image::Rgba;
use log::error;
use features::coolors::colors::ColorCount;
use features::coolors::utils::{get_colors_from, get_most_freq};

async fn get_vibrant_color(url: &str) -> Result<ColorCount, ()> {
    // fetch the image :)
    let image_buffer = match reqwest::get(url).await {
        Ok(r) => {
            match r.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(target: "Runtime", "An error occured while getting the bytes of the request at {url:?}: {e:#?}");
                    return Err(())
                }
            }
        },
        Err(e) => {
            error!(target: "Runtime", "An error occured while fetching the image at {url:?}: {e:#?}");
            return Err(())
        }
    };
    let image = image::load_from_memory(image_buffer.as_ref()).unwrap();

    let colors = get_colors_from(&image);

    let most_frequent = get_most_freq(&colors, 1);

    if most_frequent.is_empty() { return Ok(ColorCount::new(Rgba([0,0,0,0]))) };
    Ok(most_frequent[0].clone())
}


/// Error ID: 16xxx
pub(crate) mod avatar_slash {
    use image::Rgba;
    use client::manager::events::Context;
    use client::models::components::Color;
    use client::models::components::embed::{Author, Embed, EmbedImage, Footer};
    use client::models::events::InteractionCreate;
    use client::models::interaction::{ApplicationCommandOptionType, InteractionDataOptionValue};
    use client::models::message::MessageBuilder;
    use client::models::user::UserId;
    use features::coolors::colors::ColorCount;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::constants::DEFAULT_AVATAR;
    use crate::scripts::{get_guild_locale, get_user, get_user_id};
    use crate::scripts::slashs::common::get_vibrant_color;
    use crate::scripts::slashs::internal_error;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);

        let data = match &payload.interaction.data {
            Some(d) => d,
            None => return internal_error(ctx, &payload.interaction, local, "16001").await
        };

        let options = data.options.as_ref().cloned().unwrap_or(Vec::new());

        let user = options.iter()
            .find(|opt| opt.name.as_str() == "user" && opt.option_type == ApplicationCommandOptionType::User);

        match user {
            Some(u) => {
                match &u.value {
                    Some(InteractionDataOptionValue::String(user_id)) => self::user(ctx, payload, local, user_id.into()).await,
                    Some(_) => internal_error(ctx, &payload.interaction, local, "16002").await,
                    None => internal_error(ctx, &payload.interaction, local, "16003").await
                }
            },
            None => author(ctx, payload, local).await
        }
    }

    async fn author(ctx: &Context, payload: &InteractionCreate, local: String) {
        let user_id = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(u) => u,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "errors::cannot_get_user_id"))
                ).await;
                return;
            }
        };

        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let user = match get_user(ctx, &user_id).await {
            Some(u) => u,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "errors::cannot_acquire_user"))
                ).await;
                return;
            }
        };

        let vibrant_color = match user.avatar_url(128, false, "jpeg") {
            Some(url) => get_vibrant_color(url.as_str()).await
                .unwrap_or(ColorCount::new(Rgba([0,0,0,0]))),
            None => ColorCount::new(Rgba([0,0,0,0]))
        };
        let color_u8 = {
            let [r,g,b,_] = vibrant_color.rgba.0;
            (r as u64) << 16 | (g as u64) << 8 | b as u64
        };

        let avatar_link = user.avatar_url(4096, true, "png").unwrap_or(DEFAULT_AVATAR.to_string());

        let _ = payload.interaction.update(
            &ctx.skynet,
            MessageBuilder::new()
                .add_embed(
                    Embed::new()
                        .set_description(
                            message!(
                                &local,
                                "slashs::avatar::avatar_of",
                                Formatter::new()
                                    .add("link", avatar_link.as_str())
                                    .add(
                                        "name",
                                        user.global_name.clone().unwrap_or(user.username.clone())
                                    )
                            )
                        )
                        .set_image(EmbedImage::new(avatar_link))
                        .set_color(Color(color_u8))
                        .set_footer(
                            Footer::new()
                                .set_text(
                                    message!(
                                        local,
                                        "slashs::avatar::footer",
                                        Formatter::new()
                                            .add("hex", format!("{color_u8:0>6X}"))
                                    )
                                )
                        )
                )
        ).await;
    }

    async fn user(ctx: &Context, payload: &InteractionCreate, local: String, user_id: UserId) {
        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let user = match get_user(ctx, &user_id).await {
            Some(u) => u,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "errors::cannot_acquire_user"))
                ).await;
                return;
            }
        };

        let author = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => {
                get_user(ctx, &id).await
            },
            None => None
        };


        let vibrant_color = match user.avatar_url(128, false, "jpeg") {
            Some(url) => get_vibrant_color(url.as_str()).await
                .unwrap_or(ColorCount::new(Rgba([0,0,0,0]))),
            None => ColorCount::new(Rgba([0,0,0,0]))
        };
        let color_u8 = {
            let [r,g,b,_] = vibrant_color.rgba.0;
            (r as u64) << 16 | (g as u64) << 8 | b as u64
        };

        let avatar_link = user.avatar_url(4096, true, "png").unwrap_or(DEFAULT_AVATAR.to_string());

        let _ = payload.interaction.update(
            &ctx.skynet,
            MessageBuilder::new()
                .add_embed(
                    Embed::new()
                        .set_description(
                            message!(
                                &local,
                                "slashs::avatar::avatar_of",
                                Formatter::new()
                                    .add("link", avatar_link.as_str())
                                    .add(
                                        "name",
                                        user.global_name.clone().unwrap_or(user.username.clone())
                                    )
                            )
                        )
                        .set_image(EmbedImage::new(avatar_link))
                        .set_color(Color(color_u8))
                        .set_footer(
                            Footer::new()
                                .set_text(
                                    message!(
                                        local,
                                        "slashs::avatar::footer",
                                        Formatter::new()
                                            .add("hex", format!("{color_u8:0>6X}"))
                                    )
                                )
                        )
                        .set_author(
                            if let Some(author) = author {
                                Author::new()
                                    .set_icon_url(author.avatar_url(512, true, "png"))
                                    .set_name(author.global_name.unwrap_or(author.username))
                            } else {
                                Author::new()
                            }
                        )
                )
        ).await;
    }
}


/// Error ID: 17xxx
pub(crate) mod banner_slash {
    use image::Rgba;
    use client::manager::events::Context;
    use client::models::components::Color;
    use client::models::components::embed::{Author, Embed, EmbedImage, Footer};
    use client::models::events::InteractionCreate;
    use client::models::interaction::{ApplicationCommandOptionType, InteractionDataOptionValue};
    use client::models::message::MessageBuilder;
    use client::models::user::UserId;
    use features::coolors::colors::ColorCount;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::constants::DEFAULT_AVATAR;
    use crate::scripts::{get_guild_locale, get_user, get_user_id};
    use crate::scripts::slashs::common::get_vibrant_color;
    use crate::scripts::slashs::internal_error;
    use crate::crates::error_broadcaster::*;
    use crate::broadcast_error;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);

        let data = match &payload.interaction.data {
            Some(d) => d,
            None => {
                internal_error(ctx, &payload.interaction, local, "16001").await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/common.rs:banner_slash:262"),
                    interaction: BroadcastInteraction::default()
                        .set_name("banner")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("reason", "Cannot get interaction data"),
                    ctx.skynet.as_ref()
                );

                return
            }
        };

        let options = data.options.as_ref().cloned().unwrap_or(Vec::new());

        let user = options.iter()
            .find(|opt| opt.name.as_str() == "user" && opt.option_type == ApplicationCommandOptionType::User);

        match user {
            Some(u) => {
                match &u.value {
                    Some(InteractionDataOptionValue::String(user_id)) => self::user(ctx, payload, local, user_id.into()).await,
                    Some(_) => internal_error(ctx, &payload.interaction, local, "17002").await,
                    None => internal_error(ctx, &payload.interaction, local, "17003").await
                }
            },
            None => author(ctx, payload, local).await
        }
    }

    async fn author(ctx: &Context, payload: &InteractionCreate, local: String) {
        let user_id = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(u) => u,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "errors::cannot_get_user_id"))
                ).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/common.rs:banner_slash:306"),
                    interaction: BroadcastInteraction::default()
                        .set_name("banner")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("reason", "Cannot get user id"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let user = match get_user(ctx, &user_id).await {
            Some(u) => u,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "errors::cannot_acquire_user"))
                ).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/common.rs:banner_slash:334"),
                    interaction: BroadcastInteraction::default()
                        .set_name("banner")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("reason", "Cannot acquire user"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        if user.banner.is_none() {
            let _  = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(local, "slashs::banner::no_banner"))
            ).await;
            return;
        }

        let vibrant_color = match user.banner_url(128, false, "jpeg") {
            Some(url) => get_vibrant_color(url.as_str()).await
                .unwrap_or(ColorCount::new(Rgba([0,0,0,0]))),
            None => ColorCount::new(Rgba([0,0,0,0]))
        };
        let color_u8 = {
            let [r,g,b,_] = vibrant_color.rgba.0;
            (r as u64) << 16 | (g as u64) << 8 | b as u64
        };

        let banner_link = user.banner_url(4096, true, "png").unwrap_or(DEFAULT_AVATAR.to_string());

        let _ = payload.interaction.update(
            &ctx.skynet,
            MessageBuilder::new()
                .add_embed(
                    Embed::new()
                        .set_description(
                            message!(
                                &local,
                                "slashs::banner::banner_of",
                                Formatter::new()
                                    .add("link", banner_link.as_str())
                                    .add(
                                        "name",
                                        user.global_name.clone().unwrap_or(user.username.clone())
                                    )
                            )
                        )
                        .set_image(EmbedImage::new(banner_link))
                        .set_color(Color(color_u8))
                        .set_footer(
                            Footer::new()
                                .set_text(
                                    message!(
                                        local,
                                        "slashs::banner::footer",
                                        Formatter::new()
                                            .add("hex", format!("{color_u8:0>6X}"))
                                    )
                                )
                        )
                )
        ).await;
    }

    async fn user(ctx: &Context, payload: &InteractionCreate, local: String, user_id: UserId) {
        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let user = match get_user(ctx, &user_id).await {
            Some(u) => u,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "errors::cannot_acquire_user"))
                ).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("core/src/scripts/slashs/common.rs:banner_slash:418"),
                    interaction: BroadcastInteraction::default()
                        .set_name("banner")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("reason", "Cannot acquire user"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        if user.banner.is_none() {
            let _  = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(local, "slashs::banner::no_banner"))
            ).await;
            return;
        }

        let author = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => {
                get_user(ctx, &id).await
            },
            None => None
        };


        let vibrant_color = match user.banner_url(128, false, "jpeg") {
            Some(url) => get_vibrant_color(url.as_str()).await
                .unwrap_or(ColorCount::new(Rgba([0,0,0,0]))),
            None => ColorCount::new(Rgba([0,0,0,0]))
        };
        let color_u8 = {
            let [r,g,b,_] = vibrant_color.rgba.0;
            (r as u64) << 16 | (g as u64) << 8 | b as u64
        };

        let banner_link = user.banner_url(4096, true, "png").unwrap_or(DEFAULT_AVATAR.to_string());

        let _ = payload.interaction.update(
            &ctx.skynet,
            MessageBuilder::new()
                .add_embed(
                    Embed::new()
                        .set_description(
                            message!(
                                &local,
                                "slashs::banner::banner_of",
                                Formatter::new()
                                    .add("link", banner_link.as_str())
                                    .add(
                                        "name",
                                        user.global_name.clone().unwrap_or(user.username.clone())
                                    )
                            )
                        )
                        .set_image(EmbedImage::new(banner_link))
                        .set_color(Color(color_u8))
                        .set_footer(
                            Footer::new()
                                .set_text(
                                    message!(
                                        local,
                                        "slashs::banner::footer",
                                        Formatter::new()
                                            .add("hex", format!("{color_u8:0>6X}"))
                                    )
                                )
                        )
                        .set_author(
                            if let Some(author) = author {
                                Author::new()
                                    .set_icon_url(author.avatar_url(512, true, "png"))
                                    .set_name(author.global_name.unwrap_or(author.username))
                            } else {
                                Author::new()
                            }
                        )
                )
        ).await;
    }
}

pub(crate) mod welcome {
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use translation::message;
    use crate::scripts::get_guild_locale;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);
        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(message!(local, "slashs::welcome::msg"))
        ).await;
    }
}

pub(crate) mod ping {
    use std::cmp::max;
    use client::manager::events::Context;
    use client::models::components::message_components::{ActionRow, Button, ButtonStyle, Component};
    use client::models::components::Emoji;
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::scripts::get_guild_locale;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);

        let shard_manager = ctx.shard_manager.read().await;

        let mut all_latency = Vec::new();
        for (_, shard) in shard_manager.get_shards().iter() {
            let latency = *shard.ping.read().await;
            all_latency.push(latency);
        }
        // sum all latencies and divide by the number of shards
        let median = all_latency.iter().sum::<u128>() / all_latency.len() as u128;

        let actual_shard = shard_manager.get_shard(&payload.shard);
        let actual_shard_latency = if let Some(s) = actual_shard { *s.ping.read().await } else { 0 };

        let msg = {
            if actual_shard_latency == 0 {
                MessageBuilder::new().set_content(message!(local, "slashs::ping::booting"))
            } else {
                MessageBuilder::new()
                    .set_content(
                        message!(
                            local.clone(),
                            "slashs::ping::content",
                            Formatter::new()
                                .add("ping", actual_shard_latency.to_string())
                                .add("shard_id", (max(ctx.shard_id, 1)).to_string())
                        )
                    )
                    .add_component(
                        Component::ActionRow(
                            ActionRow::new().add_component(
                                Component::Button(
                                    Button::new("A")
                                        .set_label(message!(
                                            local.clone(),
                                            "slashs::ping::button::label",
                                            Formatter::new().add("median", median.to_string())
                                        ))
                                        .set_emoji(Emoji::new(None, message!(local, "slashs::ping::button::emoji")))
                                        .set_style(ButtonStyle::Secondary)
                                        .set_disabled(true)
                                )
                            )
                        )
                    )
            }
        };

        let _ = payload.interaction.reply(&ctx.skynet, msg).await;
    }
}


/// Error code: 17xxx
pub(crate) mod user_info {
    use client::manager::events::Context;
    use client::models::components::Color;
    use client::models::components::embed::Embed;
    use client::models::events::InteractionCreate;
    use client::models::interaction::{ApplicationCommandOptionType, InteractionDataOptionValue};
    use client::models::message::MessageBuilder;
    use client::models::SnowflakeInfo;
    use client::models::user::{User, UserId};
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::scripts::{get_guild_locale, get_user, get_user_id};
    use crate::scripts::slashs::internal_error;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);

        let interaction_data_options = match &payload.interaction.data {
            Some(data) => data.options.as_ref(),
            None => {
                internal_error(ctx, &payload.interaction, local, "17001").await;
                return;
            }
        };

        if let Some(options) = interaction_data_options {
            let user_id = options.iter()
                .find(|o| o.name == "user" && o.option_type == ApplicationCommandOptionType::User);

            match user_id {
                Some(user_id) => {
                    match user_id.value.as_ref() {
                        Some(InteractionDataOptionValue::String(id)) => {
                            user_id_given(
                                ctx,
                                payload,
                                local,
                                id.clone()
                            ).await;
                        },
                        _ => {
                            internal_error(ctx, &payload.interaction, local, "17003").await;
                        }
                    }
                }
                None => {
                    no_options_given(ctx, payload, local).await;
                }
            }
        } else {
            no_options_given(ctx, payload, local).await;
        }
    }

    async fn user_id_given(
        ctx: &Context,
        payload: &InteractionCreate,
        local: String,
        id: String
    ) {
        let user_id = id.into();

        let user = match get_user(ctx, &user_id).await {
            Some(u) => u,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "errors:cannot_acquire_user"))
                ).await;
                return;
            }
        };

        send_informations(
            ctx,
            payload,
            local,
            &user,
            &user_id
        ).await;
    }

    async fn no_options_given(ctx: &Context, payload: &InteractionCreate, local: String) {
        let user_id = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => id,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "errors::cannot_get_user_id"))
                ).await;
                return;
            }
        };

        let user = match get_user(ctx, &user_id).await {
            Some(u) => u,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "errors::cannot_acquire_user"))
                ).await;
                return;
            }
        };

        send_informations(
            ctx,
            payload,
            local,
            &user,
            &user_id
        ).await;
    }

    async fn send_informations(
        ctx: &Context,
        payload: &InteractionCreate,
        local: String,
        user: &User,
        user_id: &UserId
    ) {

        let snowflake_informations = match SnowflakeInfo::try_from(user_id.clone()) {
            Ok(informations) => informations,
            Err(e) => {
                internal_error(ctx, &payload.interaction, local, "17002").await;
                return;
            }
        };

        if let Some(member) = &payload.interaction.member {
            // send the member's message
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .add_embed(
                        Embed::new()
                            .set_color(Color::EMBED_COLOR)
                            .set_description(
                                message!(
                                    local,
                                    "slashs::user_info::member",
                                    Formatter::new()
                                        .add("id", user_id.to_string())
                                        .add("username", user.global_name.clone().unwrap_or(user.username.clone()))
                                        .add(
                                            "account_creation",
                                            snowflake_informations.timestamp.timestamp_millis() / 1000
                                        )
                                        .add(
                                            "join_timestamp",
                                            member.joined_at.timestamp_millis() / 1000
                                        )
                                )
                            )
                    )
            ).await;
        } else {
            // send the user's message
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .add_embed(
                        Embed::new()
                            .set_color(Color::EMBED_COLOR)
                            .set_description(
                                message!(
                                    local,
                                    "slashs::user_info::not_member",
                                    Formatter::new()
                                        .add("id", user_id.to_string())
                                        .add("username", user.global_name.clone().unwrap_or(user.username.clone()))
                                        .add(
                                            "account_creation",
                                            snowflake_informations.timestamp.timestamp_millis() / 1000
                                        )
                                )
                            )
                    )
            ).await;
        }
    }
}