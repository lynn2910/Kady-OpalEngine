use sha2::{Sha256, Digest};
use crate::constants::ADMINS;

fn is_admin(id: impl ToString) -> bool {
    // generate hash
    let hash = {
        let mut hasher = Sha256::new();
        hasher.update(id.to_string());
        hasher.finalize()
    };

    // check if hash is in admin list
    ADMINS.contains(&&hash[..])
}

mod reports {
    use chrono::Utc;
    use client::manager::http::Http;
    use client::models::channel::ChannelId;
    use client::models::components::Color;
    use client::models::components::embed::{Author, Embed};
    use client::models::message::MessageBuilder;

    pub(super) async fn report(
        http: &Http,
        author: impl ToString,
        activity_type: impl ToString,
        report: impl ToString
    )
    {
        let channel_id: ChannelId = crate::constants::ADMINS_ACTIVITY_REPORT.into();

        let _ = channel_id.send_message(
            http,
            MessageBuilder::new()
                .add_embed(
                    Embed::new()
                        .set_author(Author::new().set_name(author.to_string()))
                        .set_description(format!("> **Type:** {}\n> **Report:** {}", activity_type.to_string(), report.to_string()))
                        .set_timestamp(Utc::now())
                        .set_color(Color::from_hex("#ff174f"))
                )
        ).await;
    }
}


pub(crate) mod admin_reload_langs {
    use std::path::PathBuf;
    use std::str::FromStr;
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use config::Config;
    use translation::message;
    use crate::scripts::slashs::admin::{is_admin, reports};
    use crate::scripts::{get_guild_locale, get_user_id};

    pub(crate) async fn triggered(
        ctx: &Context,
        payload: &InteractionCreate
    )
    {
        // if the slash command isn't called from a guild (no GuildMember), we refuse the interaction
        if payload.interaction.member.is_none() {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(get_guild_locale(&payload.interaction.guild_locale), "errors::not_guild"))
            ).await;

            reports::report(
                &ctx.skynet,
                if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                "Admin command triggered (admin_reload_langs)",
                "The command 'admin_update_slashs' was triggered but the guild_member isn't accessible."
            ).await;

            return;
        }

        let user_id = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => id,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("INTERNAL ERROR")
                ).await;

                reports::report(
                    &ctx.skynet,
                    if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                    "Admin command triggered (admin_reload_langs)",
                    "The command 'admin_update_slashs' was triggered but no User ID were found."
                ).await;

                return;
            }
        };

        if !is_admin(&user_id) {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(get_guild_locale(&payload.interaction.guild_locale), "errors::admin_only"))
            ).await;

            reports::report(
                &ctx.skynet,
                if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                "Admin command triggered (admin_reload_langs)",
                format!(
                    "The command 'admin_update_slashs' was triggered by the user (above) with ID {user_id} in the channel {:?} from the guild {:?}.\n\nUser is not registered as administrator.\n\n> **Access denied successfully.**",
                    payload.interaction.channel_id,
                    payload.interaction.guild_id
                )
            ).await;

            return;
        }

        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let config = match ctx.get_data::<Config>().await {
            Some(c) => c,
            None => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("> 💥 ** ** **Une erreur est survenant en tentant d'accéder à la configuration.**")
                ).await;
                return;
            }
        };

        let langs_path = match PathBuf::from_str(config.langs.as_str()) {
            Ok(p) => p,
            Err(e) => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(format!("> 💥 ** ** **Une erreur est survenue en chargeant le chemin des langues:** {e}"))
                ).await;
                return;
            }
        };

        match translation::load_translations(langs_path.as_path()) {
            Ok(_) => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("> ✅ ** ** **Les fichiers de langues ont bien été mis à jour.**")
                ).await;
            }
            Err(e) => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(format!("> 💥 ** ** **Une erreur est survenue pendant la mise à jour des langues:** {e}"))
                ).await;
            }
        }
    }
}

pub(crate) mod admin_reload_requests {
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use config::Config;
    use database::Database;
    use translation::message;
    use crate::scripts::slashs::admin::{is_admin, reports};
    use crate::scripts::{get_guild_locale, get_user_id};

    pub(crate) async fn triggered(
        ctx: &Context,
        payload: &InteractionCreate
    )
    {
        // if the slash command isn't called from a guild (no GuildMember), we refuse the interaction
        if payload.interaction.member.is_none() {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(get_guild_locale(&payload.interaction.guild_locale), "errors::not_guild"))
            ).await;

            reports::report(
                &ctx.skynet,
                if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                "Admin command triggered (admin_reload_requests)",
                "The command 'admin_update_slashs' was triggered but the guild_member isn't accessible."
            ).await;

            return;
        }

        let user_id = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => id,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("INTERNAL ERROR")
                ).await;

                reports::report(
                    &ctx.skynet,
                    if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                    "Admin command triggered (admin_reload_requests)",
                    "The command 'admin_update_slashs' was triggered but no User ID were found."
                ).await;

                return;
            }
        };

        if !is_admin(&user_id) {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(get_guild_locale(&payload.interaction.guild_locale), "errors::admin_only"))
            ).await;

            reports::report(
                &ctx.skynet,
                if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                "Admin command triggered (admin_reload_requests)",
                format!(
                    "The command 'admin_update_slashs' was triggered by the user (above) with ID {user_id} in the channel {:?} from the guild {:?}.\n\nUser is not registered as administrator.\n\n> **Access denied successfully.**",
                    payload.interaction.channel_id,
                    payload.interaction.guild_id
                )
            ).await;

            return;
        }

        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let config = match ctx.get_data::<Config>().await {
            Some(c) => c,
            None => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("> 💥 ** ** **Une erreur est survenant en tentant d'accéder à la configuration.**")
                ).await;
                return;
            }
        };

        let mut data = ctx.data.write().await;

        let database = match data.get_mut::<Database>() {
            Some(c) => c,
            None => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("> 💥 ** ** **Une erreur est survenant en tentant d'accéder à la structure** 'Database'**.**")
                ).await;
                return;
            }
        };

        match database.update_requests(&config).await {
            Ok(_) => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("> ✅ ** ** **Les requêtes de la base de donnée ont été mise à jour.**")
                ).await;
            }
            Err(e) => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(format!("> 💥 ** ** **Une erreur est survenue:** {e}"))
                ).await;
            }
        };
    }
}

pub(crate) mod admin_reload_slashs {
    use chrono::Utc;
    use client::manager::events::Context;
    use client::models::components::Color;
    use client::models::components::embed::Embed;
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use translation::message;
    use crate::scripts::{get_guild_locale, get_user_id};
    use crate::scripts::slashs::admin::{is_admin, reports};

    pub(crate) async fn triggered(
        ctx: &Context,
        payload: &InteractionCreate
    )
    {
        // if the slash command isn't called from a guild (no GuildMember), we refuse the interaction
        if payload.interaction.member.is_none() {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(get_guild_locale(&payload.interaction.guild_locale), "errors::not_guild"))
            ).await;

            reports::report(
                &ctx.skynet,
                if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                "Admin command triggered (admin_reload_slashs)",
                "The command 'admin_update_slashs' was triggered but the guild_member isn't accessible."
            ).await;

            return;
        }

        let user_id = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => id,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("INTERNAL ERROR")
                ).await;

                reports::report(
                    &ctx.skynet,
                    if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                    "Admin command triggered (admin_reload_slashs)",
                    "The command 'admin_update_slashs' was triggered but no User ID were found."
                ).await;

                return;
            }
        };

        let channel_id = match &payload.interaction.channel_id {
            Some(id) => id,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("Cannot obtain the Channel ID")
                ) .await;
                return;
            }
        };

        if !is_admin(&user_id) {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(get_guild_locale(&payload.interaction.guild_locale), "errors::admin_only"))
            ).await;

            reports::report(
                &ctx.skynet,
                if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                "Admin command triggered (admin_reload_slashs)",
                format!(
                    "The command 'admin_update_slashs' was triggered by the user (above) with ID {user_id} in the channel {:?} from the guild {:?}.\n\nUser is not registered as administrator.\n\n> **Access denied successfully.**",
                    payload.interaction.channel_id,
                    payload.interaction.guild_id
                )
            ).await;

            return;
        }

        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let (success, errors) = crate::application_commands_manager::instance_trigger(
            ctx.skynet.clone(),
            ctx.cache.clone()
        ).await;

        if success.is_none() && errors.is_some() {
            let _ = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .add_embed(
                        Embed::new()
                            .set_color(Color::from_hex("#ff174f"))
                            .set_title("Error")
                            .set_description(format!("{} errors were received.", errors.as_ref().unwrap().len()))
                            .set_timestamp(Utc::now())
                    )
            ).await;

            for error in errors.as_ref().unwrap().iter() {
                let _ = channel_id.send_message(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(format!("```rust\n{error:#?}\n```"))
                ).await;
            }
        } else if success.is_some() && errors.is_none() {
            let _ = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .add_embed(
                        Embed::new()
                            .set_color(Color::from_hex("#17ff79"))
                            .set_timestamp(Utc::now())
                            .set_title("Success of the operation")
                            .set_description(
                                format!(
                                    "> **Everything went alright:**\n{}",
                                    success.as_ref().unwrap().iter()
                                        .map(|s| format!("- {s}"))
                                        .collect::<Vec<String>>()
                                        .join("\n")
                                )
                            )
                    )
            ).await;
        } else if success.is_none() && errors.is_some() {
            let _ = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .add_embed(
                        Embed::new()
                            .set_color(Color::from_hex("#ffd117"))
                            .set_timestamp(Utc::now())
                            .set_title("Some actions failed")
                            .set_description(
                                format!(
                                    "> {} errors was received\n> **Actions done:**\n{}\n\n:warning: For errors, they will be sent under this message",
                                    errors.as_ref().unwrap().len(),
                                    success.as_ref().unwrap().iter()
                                        .map(|s| format!("- {s}"))
                                        .collect::<Vec<String>>()
                                        .join("\n")
                                )
                            )
                    )
            ).await;

            for error in errors.as_ref().unwrap().iter() {
                let _ = channel_id.send_message(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(format!("```rust\n{error:#?}\n```"))
                ).await;
            }
        } else if success.is_none() && errors.is_none() {
            // some real shit is happening
            let _ = payload.interaction.update(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content("No informations was sent back from the app.\n\nSome real shit is happening right there, call Sedorriku NOW.")
            ).await;
        }
    }
}

pub(crate) mod admin_memory_report {
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use config::Config;
    use translation::message;
    use crate::scripts::{get_guild_locale, get_user_id};
    use crate::scripts::slashs::admin::{is_admin, reports};

    pub(crate) async fn triggered(
        ctx: &Context,
        payload: &InteractionCreate
    )
    {
        // if the slash command isn't called from a guild (no GuildMember), we refuse the interaction
        if payload.interaction.member.is_none() {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(get_guild_locale(&payload.interaction.guild_locale), "errors::not_guild"))
            ).await;

            reports::report(
                &ctx.skynet,
                if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                "Admin command triggered (admin_reload_slashs)",
                "The command 'admin_memory_report' was triggered but the guild_member isn't accessible."
            ).await;

            return;
        }

        let user_id = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => id,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("INTERNAL ERROR")
                ).await;

                reports::report(
                    &ctx.skynet,
                    if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                    "Admin command triggered (admin_reload_slashs)",
                    "The command 'admin_memory_report' was triggered but no User ID were found."
                ).await;

                return;
            }
        };

        if !is_admin(&user_id) {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(get_guild_locale(&payload.interaction.guild_locale), "errors::admin_only"))
            ).await;

            reports::report(
                &ctx.skynet,
                if let Some(u) = &payload.interaction.user { format!("{:?} ({})", u.global_name, u.username) } else { "unknown".to_string() },
                "Admin command triggered (admin_reload_slashs)",
                format!(
                    "The command 'admin_memory_report' was triggered by the user (above) with ID {user_id} in the channel {:?} from the guild {:?}.\n\nUser is not registered as administrator.\n\n> **Access denied successfully.**",
                    payload.interaction.channel_id,
                    payload.interaction.guild_id
                )
            ).await;

            return;
        }

        let config = match ctx.get_data::<Config>().await {
            Some(c) => c,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content("> ❌ ** ** **Cannot get the object** `Config` **from the context.**")
                ).await;
                return;
            }
        };

        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let report = crate::tasks::report_memory_usage(
            ctx.cache.clone(),
            ctx.shard_manager.clone(),
            config,
            // get Kb
            false
        ).await;

        match report {
            Err(e) => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(format!("> :x: ** ** **Une erreur est survenue:** {e}"))
                ).await;
            },
            Ok(report) => {
                let _ = payload.interaction.update(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(format!("> :white_check_mark: ** ** **Rapport d'utilisation de la mémoire:**\n```rs\n{report:#?}\n```"))
                ).await;
            }
        }
    }
}