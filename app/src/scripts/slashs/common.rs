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
                        .set_code_path("app/src/scripts/slashs/common.rs:banner_slash:262"),
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
                        .set_code_path("app/src/scripts/slashs/common.rs:banner_slash:306"),
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
                        .set_code_path("app/src/scripts/slashs/common.rs:banner_slash:334"),
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
                        .set_code_path("app/src/scripts/slashs/common.rs:banner_slash:418"),
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
    use client::models::components::embed::{Author, Embed, Thumbnail};
    use client::models::events::InteractionCreate;
    use client::models::interaction::{ApplicationCommandOptionType, InteractionDataOptionValue};
    use client::models::message::MessageBuilder;
    use client::models::SnowflakeInfo;
    use client::models::user::{User, UserId};
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::constants::DEFAULT_AVATAR;
    use crate::scripts::{get_guild_locale, get_member, get_user, get_user_id};
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
            Err(_) => {
                internal_error(ctx, &payload.interaction, local, "17002").await;
                return;
            }
        };

        let author = get_user(
            &ctx,
            &get_user_id(
                &payload.interaction.user,
                &payload.interaction.member
            ).unwrap_or(String::new().into())
        ).await;

        let member = if let Some(guild_id) = &payload.interaction.guild_id {
            get_member(&ctx, &guild_id, &user.id).await
        } else {
            None
        };

        if let Some(member) = member {
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
                            ).set_thumbnail(
                            Thumbnail::new(
                                user.avatar_url(4096, true, "png").unwrap_or(DEFAULT_AVATAR.to_string())
                            )
                        ).set_author(
                            Author::new()
                                .set_name(
                                    author.as_ref().map(|auth|
                                        auth.global_name
                                            .clone()
                                            .unwrap_or(auth.username.clone()))
                                        .unwrap_or("Unknown author".to_string())
                                )
                                .set_icon_url(
                                    author.map(|auth|
                                        auth.avatar_url(256, false, "png")
                                            .unwrap_or(DEFAULT_AVATAR.to_string()))
                                )
                        )
                    )
            ).await;
        } else {
            let author = get_user(
                &ctx,
                &get_user_id(
                    &payload.interaction.user,
                    &payload.interaction.member
                ).unwrap_or(String::new().into())
            ).await;


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
                            ).set_thumbnail(
                            Thumbnail::new(
                                    user.avatar_url(4096, true, "png")
                                        .unwrap_or(DEFAULT_AVATAR.to_string())
                                )
                            ).set_author(
                                Author::new()
                                    .set_name(
                                        author.as_ref().map(|auth|
                                            auth.global_name
                                                .clone()
                                                .unwrap_or(auth.username.clone()))
                                            .unwrap_or("Unknown author".to_string())
                                    )
                                    .set_icon_url(
                                        author.map(|auth|
                                            auth.avatar_url(256, false, "png")
                                                .unwrap_or(DEFAULT_AVATAR.to_string()))
                                    )
                            )
                    )
            ).await;
        }
    }
}

/// Error code: 18xxx
pub(crate) mod help {
    use log::error;
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;
    use client::models::interaction::ApplicationCommandOptionType;
    use client::models::message::MessageBuilder;
    use translation::message;
    use crate::application_commands_manager::{COMMANDS, get_command_type};
    use crate::assets::help;
    use crate::scripts::get_guild_locale;
    use crate::scripts::slashs::internal_error;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);

        let interaction_data = payload.interaction.data.as_ref()
            .unwrap();

        if let Some(command) = interaction_data.options.as_ref().unwrap_or(&Vec::new())
            .iter()
            .find(|opt| opt.name == "command" && opt.option_type == ApplicationCommandOptionType::String && opt.value.is_some())
            .map(|opt| opt.value.clone().unwrap())
        {
            let name = command.to_string();

            let local = get_guild_locale(&payload.interaction.guild_locale);

            let commands = COMMANDS.read().await;
            if let Some(cmd_type) = get_command_type(&commands, name.as_str()) {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    help::generate_command_help(
                        &ctx,
                        cmd_type,
                        name.as_str(),
                        &local
                    ).await
                ).await;
                return;
            } else {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "slashs::help::command::unknown"))
                ).await;
                return;
            }

        }

        match help::generate_default_message(ctx, &local).await {
            Ok(msg) => {
                let _ = payload.interaction.reply(&ctx.skynet, msg).await;
            },
            Err((code, error)) => {
                error!(target: "Runtime", "An error occured in the help command (code {code}): {error}");
                internal_error(ctx, &payload.interaction, local, format!("18{code:03}")).await;
            }
        }
    }
}


pub(crate) mod kady {
    use client::manager::events::Context;
    use client::models::components::embed::{Author, Embed, Footer};
    use client::models::components::Emoji;
    use client::models::components::message_components::{ActionRow, Button, ButtonStyle, Component, ComponentType, TextInput, TextInputStyle};
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use config::Config;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use client::models::components::Color;
    use crate::{constants, CoreStart};
    use crate::constants::DEFAULT_AVATAR;
    use crate::scripts::{get_client_user, get_guild_locale};
    use crate::scripts::slashs::internal_error;

    const AVAILABLE_SUBCOMMANDS: &[&str] = &["invite", "support", "informations", "suggest", "issue", "review"];

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let interaction_data = payload.interaction.data.as_ref()
            .unwrap();

        let local = get_guild_locale(&payload.interaction.guild_locale);

        let options = interaction_data.options.clone()
            .unwrap_or(Vec::new());

        let subcommand = options.iter()
            .find(|opt|
                AVAILABLE_SUBCOMMANDS.contains(&opt.name.as_str()));

        match subcommand {
            Some(sub) => {
                match sub.name.as_str() {
                    "invite" => invite(ctx, payload, &local).await,
                    "support" => support(ctx, payload, &local).await,
                    "informations" => informations(ctx, payload, &local).await,
                    "suggest" => suggest(ctx, payload, &local).await,
                    "issue" => report_issue(ctx, payload, &local).await,
                    "review" => review(ctx, payload, &local).await,
                    _ => {
                        let _ = payload.interaction.reply(
                            &ctx.skynet,
                            MessageBuilder::new()
                                .set_content(message!(local, "slashs::kady::invalid_subcommand"))
                        ).await;
                    }
                }
            },
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "slashs::kady::no_subcommand"))
                ).await;
            }
        }
    }


    async fn support(ctx: &Context, payload: &InteractionCreate, local: &String) {
        let config: Config = match ctx.get_data().await {
            Some(c) => c,
            None => return internal_error(ctx, &payload.interaction, local, "18003").await
        };

        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(message!(local, "slashs::kady::support::desc"))
                .add_component(
                    Component::ActionRow(
                        ActionRow::new()
                            .add_component(
                                Component::Button(
                                    Button::new("")
                                        .set_url(config.client.support_url)
                                        .set_style(ButtonStyle::Link)
                                        .set_label(message!(local, "slashs::kady::support::btn"))
                                        .set_emoji(Emoji::new(None, "🔗"))
                                )
                            )
                    )
                )
        ).await;
    }

    async fn invite(ctx: &Context, payload: &InteractionCreate, local: &String) {
        let client_user = match get_client_user(ctx).await {
            Some(c) => c,
            None => return internal_error(ctx, &payload.interaction, local, "18001").await
        };

        let config: Config = match ctx.get_data().await {
            Some(c) => c,
            None => return internal_error(ctx, &payload.interaction, local, "18002").await
        };

        let invite_url = constants::generate_invite_link(&client_user.id, &config);

        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(message!(local, "slashs::kady::invite::desc"))
                .add_component(
                    Component::ActionRow(
                        ActionRow::new()
                            .add_component(
                                Component::Button(
                                    Button::new("")
                                        .set_url(invite_url)
                                        .set_style(ButtonStyle::Link)
                                        .set_label(message!(local, "slashs::kady::invite::btn"))
                                        .set_emoji(Emoji::new(None, "🔗"))
                                )
                            )
                    )
                )
        ).await;
    }

    async fn informations(ctx: &Context, payload: &InteractionCreate, local: &String) {
        let config: Config = match ctx.get_data().await {
            Some(c) => c,
            None => return internal_error(ctx, &payload.interaction, local, "18004").await
        };
        let client_user = get_client_user(ctx).await;

        let client_start_time = match ctx.get_data::<CoreStart>().await {
            Some(d) => d.0.timestamp(),
            None => 0
        };

        let cache = ctx.cache.read().await;
        let guild_count = cache.get_guild_size();
        let channels_count = cache.get_channel_size();
        drop(cache);

        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .add_embed(
                    Embed::new()
                        .set_author(
                            Author::new()
                                .set_name(
                                    format!(
                                        "{} Informations",
                                        client_user.as_ref().map(|c|
                                            c.global_name.as_ref().unwrap_or(&c.username))
                                            .unwrap_or(&"Skynet".to_string())
                                    )
                                )
                                .set_icon_url(
                                    client_user.as_ref().map(|c| c.avatar_url(512, false, "png"))
                                        .unwrap_or(Some(DEFAULT_AVATAR.to_string()))
                                )
                        )
                        .set_footer(
                            Footer::new()
                                .set_text(
                                    message!(
                                        local,
                                        "const::copyright"
                                    )
                                )
                        )
                        .set_color(
                            Color::from_hex(
                                message!(
                                    local,
                                    "const::palette::main"
                                )
                            )
                        )
                        .set_description(
                            message!(
                                local,
                                "slashs::kady::informations::desc",
                                Formatter::new()
                                    .add("cargo", config.core.cargo_version)
                                    .add("rustup", config.core.rustup_version)
                                    .add("rustc", config.core.rustc_version)
                                    .add("online_since", client_start_time)
                                    .add("support", config.client.support_url)
                                    .add("website", config.client.website)
                                    .add("top_gg", config.client.top_gg)
                                    .add("servers", guild_count)
                                    .add("channels", channels_count)
                            )
                        )
                )
        ).await;
    }

    async fn suggest(ctx: &Context, payload: &InteractionCreate, local: &String) {
        let _ = payload.interaction.reply_with_modal(
            &ctx.skynet,
            MessageBuilder::new()
                .set_title(message!(local, "slashs::kady::suggest::title"))
                .set_custom_id("KADY_SUGGEST_MODAL")
                .add_component(
                    Component::ActionRow(
                        ActionRow::new()
                            .add_component(
                                Component::TextInput(
                                    TextInput {
                                        kind: ComponentType::TextInput,
                                        style: Some(TextInputStyle::Paragraph),
                                        label: Some(message!(local, "slashs::kady::suggest::question").into()),
                                        custom_id: "SUGGESTION".into(),
                                        placeholder: Some(message!(local, "slashs::kady::suggest::question_placeholder").into()),
                                        min_length: 1.into(),
                                        max_length: 1024.into(),
                                        disabled: Some(false),
                                        value: None,
                                        required: true
                                    }
                                )
                            )
                    )
                ).add_component(
                Component::ActionRow(
                    ActionRow::new()
                        .add_component(
                            Component::TextInput(
                                TextInput {
                                    kind: ComponentType::TextInput,
                                    style: Some(TextInputStyle::Paragraph),
                                    label: Some(message!(local, "slashs::kady::suggest::contacted_after").into()),
                                    custom_id: "SUGGESTION_CONTACT_AFTER".into(),
                                    placeholder: Some("Y/N - O/N".into()),
                                    min_length: 1.into(),
                                    max_length: 4.into(),
                                    disabled: Some(false),
                                    value: None,
                                    required: false
                                }
                            )
                        )
                )
            )
        ).await;
    }

    async fn report_issue(ctx: &Context, payload: &InteractionCreate, local: &String) {
        let _ = payload.interaction.reply_with_modal(
            &ctx.skynet,
            MessageBuilder::new()
                .set_title(message!(local, "slashs::kady::issue::title"))
                .set_custom_id("KADY_ISSUE_MODAL")
                .add_component(
                    Component::ActionRow(
                        ActionRow::new()
                            .add_component(
                                Component::TextInput(
                                    TextInput {
                                        kind: ComponentType::TextInput,
                                        style: Some(TextInputStyle::Paragraph),
                                        label: Some(message!(local, "slashs::kady::issue::bug_type").into()),
                                        custom_id: "BUG_TYPE".into(),
                                        placeholder: Some(message!(local, "slashs::kady::issue::bug_type_placeholder").into()),
                                        min_length: 1.into(),
                                        max_length: 256.into(),
                                        disabled: Some(false),
                                        value: None,
                                        required: true
                                    }
                                )
                            )
                    )
                ).add_component(
                Component::ActionRow(
                    ActionRow::new()
                        .add_component(
                            Component::TextInput(
                                TextInput {
                                    kind: ComponentType::TextInput,
                                    style: Some(TextInputStyle::Paragraph),
                                    label: Some(message!(local, "slashs::kady::issue::bug_description").into()),
                                    custom_id: "BUG_DESCRIPTION".into(),
                                    placeholder: Some(message!(local, "slashs::kady::issue::bug_desc_placeholder").into()),
                                    min_length: 1.into(),
                                    max_length: 1024.into(),
                                    disabled: Some(false),
                                    value: None,
                                    required: true
                                }
                            )
                        )
                )
            ).add_component(
                Component::ActionRow(
                    ActionRow::new()
                        .add_component(
                            Component::TextInput(
                                TextInput {
                                    kind: ComponentType::TextInput,
                                    style: Some(TextInputStyle::Paragraph),
                                    label: Some(message!(local, "slashs::kady::issue::contacted_after").into()),
                                    custom_id: "SUGGESTION_CONTACT_AFTER".into(),
                                    placeholder: Some("Y/N - O/N".into()),
                                    min_length: 1.into(),
                                    max_length: 4.into(),
                                    disabled: Some(false),
                                    value: None,
                                    required: false
                                }
                            )
                        )
                )
            )
        ).await;
    }

    async fn review(ctx: &Context, payload: &InteractionCreate, local: &String) {
        let _ = payload.interaction.reply_with_modal(
            &ctx.skynet,
            MessageBuilder::new()
                .set_title(message!(local, "slashs::kady::review::title"))
                .set_custom_id("KADY_REVIEW_MODAL")
                .add_component(
                    Component::ActionRow(
                        ActionRow::new()
                            .add_component(
                                Component::TextInput(
                                    TextInput {
                                        kind: ComponentType::TextInput,
                                        style: Some(TextInputStyle::Paragraph),
                                        label: Some(message!(local, "slashs::kady::review::review").into()),
                                        custom_id: "REVIEW".into(),
                                        placeholder: Some(message!(local, "slashs::kady::review::review_placeholder").into()),
                                        min_length: 1.into(),
                                        max_length: 256.into(),
                                        disabled: Some(false),
                                        value: None,
                                        required: true
                                    }
                                )
                            )
                    )
                ).add_component(
                Component::ActionRow(
                    ActionRow::new()
                        .add_component(
                            Component::TextInput(
                                TextInput {
                                    kind: ComponentType::TextInput,
                                    style: Some(TextInputStyle::Paragraph),
                                    label: Some(message!(local, "slashs::kady::review::note").into()),
                                    custom_id: "NOTE".into(),
                                    placeholder: Some(message!(local, "slashs::kady::review::note_placeholder").into()),
                                    min_length: 1.into(),
                                    max_length: 5.into(),
                                    disabled: Some(false),
                                    value: None,
                                    required: true
                                }
                            )
                        )
                )
            )
        ).await;
    }
}