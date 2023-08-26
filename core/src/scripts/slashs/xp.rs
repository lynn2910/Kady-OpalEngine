pub(crate) mod guild_rank {
    use std::io::Cursor;
    use log::error;
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;
    use client::models::interaction::InteractionDataOptionValue;
    use client::models::message::{AttachmentBuilder, MessageAttachmentBuilder, MessageBuilder};
    use database::Database;
    use database::model::guild::{Guild, GuildUserXp, UserXpRank};
    use database::model::users::User;
    use features::xp;
    use features::xp::image_gen::FontContainer;
    use translation::message;
    use crate::scripts::get_guild_locale;
    use crate::scripts::slashs::{internal_error_deferred};

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
        let pool = database.get_pool().await;
        let requests = database.get_requests().await;

        let guild_data = {
            match Guild::from_pool(&pool, requests.guilds.get.as_str(), guild_id).await {
                Ok(g) => g,
                Err(e) => {
                    error!(target: "Runtime", "An error occured while acquiring the guild informations: {e:#?}");
                    cannot_get_guild_data(ctx, payload).await;
                    return;
                }
            }
        };

        // check if the functionality is enabled
        if !guild_data.xp_enabled.unwrap_or(false) {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(local, "features::xp::disabled"))
                    .set_ephemeral(true)
            ).await;
            return;
        }

        let user_id = if let Some(data) = &payload.interaction.data {
            let r: Option<String> = if let Some(options) = &data.options {
                let opt = options.iter().find(|o| o.name == "user");

                if let Some(option) = opt {
                    match &option.value {
                        Some(InteractionDataOptionValue::String(v)) => Some(v.to_string()),
                        Some(_) => None,
                        None => None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            match r {
                Some(v) => Some(v),
                None => {
                    if let Some(u) = &payload.interaction.user {
                        Some(u.id.to_string())
                    } else if let Some(g) = &payload.interaction.member {
                        g.user.as_ref().map(|u| u.id.to_string())
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        };

        if user_id.is_none() {
            return cannot_acquire_user(ctx, payload).await;
        }
        let user_id = user_id.unwrap();

        // defer the message response
        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        // check if the user is a member
        {
            match ctx.skynet.fetch_guild_member(guild_id, &user_id.clone().into()).await {
                Ok(Ok(_)) => (),
                Ok(Err(_)) => {
                    let _ = payload.interaction.update(
                        &ctx.skynet,
                        MessageBuilder::new()
                            .set_content(message!(guild_data.lang, "errors::not_a_member"))
                    ).await;
                    return;
                }
                Err(_) => { return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13011").await }
            }
        }

        let user = match ctx.skynet.fetch_user(&user_id).await {
            Ok(user) => match user {
                Ok(u) => u,
                Err(e) => {
                    error!(target: "Runtime", "An error occured while fetching a user for the guild_rank system: {e:#?}");
                    cannot_acquire_user(ctx, payload).await;
                    return;
                }
            },
            Err(e) => {
                error!(target: "Runtime", "An error occured while trying to fetch a user for the guild_rank system: {e:#?}");
                cannot_acquire_user(ctx, payload).await;
                return;
            }
        };

        if user.bot.unwrap_or(false) {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(guild_data.lang, "errors::not_for_bot"))
            ).await;
            return;
        }

        // ensure the presence of the user in the database
        {
            match User::ensure(&pool, requests.users.ensure.as_str(), user_id.to_string()).await {
                Ok(()) => (),
                Err(e) => {
                    error!(target: "Runtime", "An error occured while ensuring the presence of the author in the database from the guild_rank command: {e:#?}")
                }
            }
        }

        // get the xp informations
        {
            match GuildUserXp::ensure(&pool, requests.guilds.xp.ensure.as_str(), &guild_data.id, &user.id).await {
                Ok(_) => (),
                Err(_) => return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13001").await
            };
        }

        let xp_data = match GuildUserXp::from_pool(&pool, requests.guilds.xp.get.as_str(), &guild_data.id, &user.id).await {
            Ok(d) => d,
            Err(_) => return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13002").await
        };

        let font_container = match ctx.get_data::<FontContainer>().await {
            Some(e) => e,
            None => return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13003").await
        };

        let rank = match sqlx::query_as::<_, UserXpRank>(requests.guilds.xp.get_rank.as_str())
            .bind(guild_id.to_string())
            .bind(user_id.to_string())
            .fetch_one(&pool.to_owned()).await
        {
            Ok(q) => q,
            Err(e) => {
                error!(target: "Runtime", "An error occured while obtaining the rank of the user for the guild xp rank: {e:#?}");
                return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13004").await;
            }
        };

        let xp_algo = xp::AlgorithmsSuites::from(guild_data.xp_algo.unwrap_or(0));

        let guild_name = {
            let cache = ctx.cache.read().await;

            if let Some(g) = cache.get_guild(guild_id) { g.name.clone() }
            else {
                drop(cache);

                match ctx.skynet.fetch_guild(guild_id).await {
                    Ok(Ok(g)) => g.name.clone(),
                    Ok(Err(e)) => {
                        error!(target: "Runtime", "An error occured because of fetching the Guild for the guild_rank command: {e:#?}");
                        return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13005").await;
                    }
                    Err(e) => {
                        error!(target: "Runtime", "An error occured while trying to fetch the Guild for the guild_rank command: {e:#?}");
                        return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13006").await;
                    }
                }
            }
        };

        let user_avatar_url = user.avatar_url(256, false, "png").unwrap_or(crate::constants::DEFAULT_AVATAR.to_string());

        let avatar = match reqwest::get(user_avatar_url).await {
            Ok(res) => match res.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(target: "Runtime", "An error occured while transforming the response into bytes: {e:#}");
                    return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13007").await
                }
            },
            Err(e) => {
                error!(target: "Runtime", "An error occured while fetching the user's avatar: {e:#}");
                return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13008").await
            }
        };

        let image = xp::image_gen::gen_guild_image(
            &user.global_name.unwrap_or(user.username),
            avatar.as_ref().to_vec(),
            xp_data.xp,
            &guild_name,
            rank.rn as u32,
            message!(guild_data.lang.clone(), "features::xp::rank").to_string(),
            &font_container,
            xp_algo
        ).await;

        match image {
            Ok(img) => {
                let mut bytes: Vec<u8> = Vec::new();
                if let Err(e) = img.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png) {
                    error!(target: "Runtime", "An error occured while manipulating the bytes of the xp guild rank image: {e:#?}");
                    internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13010").await;
                };

                let file = AttachmentBuilder {
                    bytes,
                    content_type: "image/png".into(),
                    description: None,
                    filename: "card.png".into(),
                    id: 0
                };

                let msg = MessageBuilder::new()
                    .add_attachment(MessageAttachmentBuilder {
                        name: "card.png".into(),
                        description: None,
                        content_type: "image/png".into(),
                        id: 0
                    });

                let m = payload.interaction.update_with_files(
                    &ctx.skynet,
                    msg,
                    vec![file]
                ).await;

                if let Err(e) = m {
                    dbg!(&e);
                }
            },
            Err(e) => {
                error!(target: "Runtime", "An error occured while generating the guild xp card: {e:#?}");
                internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13009").await;
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

    async fn cannot_acquire_user(ctx: &Context, payload: &InteractionCreate) {
        let guild_locale = get_guild_locale(&payload.interaction.guild_locale);
        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(message!(guild_locale, "errors::cannot_acquire_user"))
                .set_ephemeral(true)
        ).await;
    }
}