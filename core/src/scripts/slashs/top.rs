//! Error ID: 14xxx

use client::manager::events::Context;
use client::models::events::InteractionCreate;
use client::models::message::MessageBuilder;
use translation::message;
use crate::scripts::get_guild_locale;
use crate::scripts::slashs::internal_error;

const AVAILABLE_CATEGORIES: &[&str] = &["cookies", "xp"];

pub(crate) async fn triggered(
    ctx: &Context,
    payload: &InteractionCreate
)
{
    let local = get_guild_locale(&payload.interaction.guild_locale);

    let data = if let Some(d) = &payload.interaction.data {
        d
    } else {
        return internal_error(ctx, &payload.interaction, local, "14001").await
    };

    if data.options.is_none() {
        return internal_error(ctx, &payload.interaction, local, "14002").await
    }
    let options = data.options.as_ref().unwrap();

    let subcommand = options.iter().find(|opt| AVAILABLE_CATEGORIES.contains(&opt.name.as_str()));

    match subcommand {
        Some(sub) => {
            match sub.name.as_str() {
                "xp" => categories::xp(ctx, payload, local).await,
                "cookies" => categories::cookies_global(ctx, payload, local).await,
                _ => {
                    let _ = payload.interaction.reply(
                        &ctx.skynet,
                        MessageBuilder::new()
                            .set_content(message!(local, "engagement::top::invalid_category"))
                    ).await;
                }
            }
        },
        None => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(local, "engagement::top::no_category"))
            ).await;
        }
    }
}

mod categories {
    use log::error;
    use client::manager::events::Context;
    use client::models::components::Color;
    use client::models::components::embed::{Author, Embed, Thumbnail};
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use database::Database;
    use database::model::guild::{Guild, GuildUserXp, UserXpRank};
    use database::model::users::CookieTopRank;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::scripts::{get_client_user, get_guild, get_user, get_user_id};
    use crate::scripts::slashs::{internal_error, internal_error_deferred};
    use crate::scripts::slashs::top::cannot_get_guild_data;

    pub(super) async fn xp(
        ctx: &Context,
        payload: &InteractionCreate,
        local: String
    )
    {
        let guild_id = match &payload.interaction.guild_id {
            Some(id) => id,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new().set_content(message!(local, "engagement::top::guild_only"))
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

        if !guild_data.xp_enabled.unwrap_or(false) {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(guild_data.lang, "engagement::top::xp_disabled"))
            ).await;
            return;
        }

        let algo_suite = features::xp::AlgorithmsSuites::from(guild_data.xp_algo.unwrap_or(0));

        let mut top_10 = match GuildUserXp::get_top_10(&pool, requests.guilds.xp.get_top_10.as_str(), guild_id).await {
            Ok(rankings) => rankings,
            Err(e) => {
                error!(target: "Runtime", "An error occured while querying the top 10 xp: {e:#?}");
                return internal_error(ctx, &payload.interaction, guild_data.lang, "14003").await
            }
        };

        if top_10.is_empty() {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(guild_data.lang, "engagement::top::empty"))
                    .set_ephemeral(true)
            ).await;
            return;
        }

        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        // sorting the vector
        top_10.sort_by_key(|r| r.xp);

        // this will be the ranking list
        let mut rankings = String::new();

        for (index, ranking) in top_10.iter().rev().enumerate() {
            let lvl = features::xp::calc_lvl(algo_suite, ranking.xp as f64);

            let user = get_user(ctx, &ranking.user_id).await;

            if index > 0 { rankings.push('\n') }

            rankings.push_str(
                message!(
                    guild_data.lang.clone(),
                    "engagement::top::xp_style",
                    Formatter::new()
                        .add("rank", index + 1)
                        .add(
                            "name",
                            user.map(|u| u.global_name.unwrap_or(ranking.user_id.to_string())).unwrap_or(ranking.user_id.to_string())
                        )
                        .add("xp", ranking.xp)
                        .add("lvl", lvl)
                ).to_string().as_str()
            )
        };

        // add the author rank
        let author_rank = {
            let author_id = get_user_id(&payload.interaction.user, &payload.interaction.member);

            if let Some(user_id) = author_id {
                match sqlx::query_as::<_, UserXpRank>(requests.guilds.xp.get_rank.as_str())
                    .bind(guild_id.to_string())
                    .bind(user_id.to_string())
                    .fetch_one(&pool.to_owned()).await
                {
                    Ok(q) => Some(q.rn),
                    Err(e) => {
                        error!(target: "Runtime", "An error occured while obtaining the rank of the user for the guild xp rank: {e:#?}");
                        return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13004").await;
                    }
                }
            } else {
                None
            }
        };

        rankings.push_str("\n\n");
        rankings.push_str(
            message!(
                guild_data.lang.clone(),
                "engagement::top::xp_author_rank",
                Formatter::new().add("rank", author_rank.unwrap_or(-1))
            ).to_string().as_str()
        );

        let guild = get_guild(ctx, guild_id).await;

        let msg = MessageBuilder::new()
            .set_content(
                message!(
                    guild_data.lang.clone(),
                    "engagement::top::xp_top",
                    Formatter::new().add("guild", guild.as_ref().map(|g| g.name.clone()).unwrap_or("UnknownGuild".to_string()))
                )
            )
            .add_embed(
                Embed::new()
                    .set_color(Color::from_hex(message!(guild_data.lang, "const::palette::embed")))
                    .set_thumbnail(
                        Thumbnail::new(
                            guild
                                    .map(
                                        |g|
                                            g.icon_url(512, true, "png")
                                                .unwrap_or(crate::constants::DEFAULT_AVATAR.to_string())
                                    )
                                    .unwrap_or(crate::constants::DEFAULT_AVATAR.to_string())
                        )
                    )
                    .set_description(rankings)
            );

        let _ = payload.interaction.update(&ctx.skynet, msg).await;
    }

    pub(super) async fn cookies_global(
        ctx: &Context,
        payload: &InteractionCreate,
        local: String
    )
    {
        let guild_id = match &payload.interaction.guild_id {
            Some(g) => g,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(message!(local, "engagement::top::global::no_dm"))
                        .set_ephemeral(true)
                ).await;
                return;
            }
        };

        let database = ctx.get_data::<Database>().await.expect("Cannot acquire the database structure");
        let pool = database.get_pool().await;
        let requests = database.get_requests().await;

        let guild_data = match Guild::from_pool(&pool, requests.guilds.get.as_str(), guild_id).await {
                Ok(g) => g,
                Err(e) => {
                    error!(target: "Runtime", "An error occured while acquiring the guild informations: {e:#?}");
                    cannot_get_guild_data(ctx, payload).await;
                    return;
                }
            };

        let mut top_10 = {
            let query = sqlx::query_as::<_, database::model::users::CookieRanking>(requests.users.cookies.get_top_10_global.as_str());

            match query.fetch_all(&pool.to_owned()).await {
                Ok(rankings) => rankings,
                Err(e) => {
                    error!(target: "Runtime", "An error occured while querying the top 10 xp: {e:#?}");
                    return internal_error(ctx, &payload.interaction, guild_data.lang, "14005").await;
                }
            }
        };

        if top_10.is_empty() {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(guild_data.lang, "engagement::top::empty"))
                    .set_ephemeral(true)
            ).await;
            return;
        }

        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        // sorting the vector
        top_10.sort_by_key(|r| r.cookies);


        // this will be the ranking list
        let mut rankings = String::new();

        for (index, ranking) in top_10.iter().rev().enumerate() {
            let id = ranking.user_to.clone().into();
            let user = get_user(ctx, &id).await;

            if index > 0 { rankings.push('\n') }

            rankings.push_str(
                message!(
                    guild_data.lang.clone(),
                    "engagement::top::cookies_style",
                    Formatter::new()
                        .add("rank", index + 1)
                        .add(
                            "name",
                            user.map(|u| u.global_name.unwrap_or(id.to_string())).unwrap_or(id.to_string())
                        )
                        .add("cookies", ranking.cookies)
                ).to_string().as_str()
            )
        };




        // add the author rank
        let author_rank = {
            let author_id = get_user_id(&payload.interaction.user, &payload.interaction.member);

            if let Some(user_id) = author_id {
                match sqlx::query_as::<_, CookieTopRank>(requests.users.cookies.get_user_rank_global.as_str())
                    .bind(user_id.to_string())
                    .fetch_optional(&pool.to_owned()).await
                {
                    Ok(q) => {
                        dbg!(&q);
                        q.map(|q| q.user_rank)
                    },
                    Err(e) => {
                        error!(target: "Runtime", "An error occured while obtaining the rank of the user for the global cookies rank: {e:#?}");
                        return internal_error_deferred(ctx, &payload.interaction, guild_data.lang, "13006").await;
                    }
                }
            } else {
                None
            }
        };

        rankings.push_str("\n\n");
        if let Some(rank) = author_rank {
            rankings.push_str(
                message!(
                    guild_data.lang.clone(),
                    "engagement::top::cookies_author_rank",
                    Formatter::new().add("rank", rank)
                ).to_string().as_str()
            );
        } else {
            rankings.push_str(
                message!(
                    guild_data.lang.clone(),
                    "engagement::top::no_cookies"
                ).to_string().as_str()
            );
        }

        let application = get_client_user(ctx).await;

        let msg = MessageBuilder::new()
            .set_content(message!(guild_data.lang.clone(),"engagement::top::cookies_top_global"))
            .add_embed(
                Embed::new()
                    .set_color(Color::from_hex(message!(guild_data.lang, "const::palette::embed")))
                    .set_author(
                        Author::new()
                            .set_icon_url(
                                Some(
                                    application
                                        .as_ref()
                                        .map(
                                            |app|
                                                app.avatar_url(512, false, "png")
                                                    .unwrap_or(crate::constants::DEFAULT_AVATAR.to_string())
                                        )
                                        .unwrap_or(crate::constants::DEFAULT_AVATAR.to_string())
                                )
                            )
                            .set_name(
                                application
                                    .map(|app|
                                        app.global_name.unwrap_or(app.username)
                                    ).unwrap_or("Skynet".to_string())
                            )
                    )
                    .set_description(rankings)
            );

        let _ = payload.interaction.update(&ctx.skynet, msg).await;
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