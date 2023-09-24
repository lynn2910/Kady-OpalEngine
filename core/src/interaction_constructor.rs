use std::sync::Arc;
use log::{error, info};
use tokio::sync::RwLock;
use client::manager::cache::CacheManager;
use client::manager::http::Http;
use client::models::interaction::{ApplicationCommand, ApplicationCommandOption, ApplicationCommandOptionType, ApplicationCommandType};
use error::RuntimeError;

#[allow(dead_code)]

/// This function will send a post request for all scripts
///
/// Don't call too much or you will get rate-limited very badly
pub(crate) async fn instance_trigger(
    http: Arc<Http>,
    cache_clone: Arc<RwLock<CacheManager>>
) -> (Option<Vec<String>>, Option<Vec<RuntimeError>>)
{
    let application = {
        // try to get it from the cache, else we need to fetch it (and add to the cache)
        let cache = cache_clone.read().await;
        if let Some(application) = cache.get_application() {
            application.clone()
        } else {
            let application = match http.fetch_application().await.unwrap() {
                Ok(application) => application,
                Err(e) => {
                    error!(target: "InteractionUpdater", "Failed to fetch application: {:?}", e);
                    return (
                        None,
                        Some(
                            vec![
                                RuntimeError::new(e.to_string())
                                    .with_context("Failed to fetch application")
                                    .with_target("fetching_application")
                            ]
                        )
                    );
                }
            };
            let mut cache = cache_clone.write().await;
            cache.update_application(&application);
            application
        }
    };


    let mut success: Vec<String> = vec!["fetching_application".to_string()];
    let mut errors: Vec<RuntimeError> = Vec::new();


    // first, delete all global commands
    {
        if let Ok(commands) = http.get_global_commands(&application.id).await {
            let commands = match commands {
                Ok(commands) => commands,
                Err(e) => {
                    error!(target: "InteractionUpdater", "Failed to get global commands: {:?}", e);
                    return (
                        Some(success),
                        Some(vec![
                            RuntimeError::new(e.to_string())
                                .with_target("get_global_commands")
                                .with_context("Failed to get global commands")
                        ])
                    );
                }
            };

            println!("Deleting {} global commands", commands.len());
            for command in commands {
                let r = http.delete_global_command(&application.id, &command.id).await;
                if let Err(e) = r {
                    error!(target: "InteractionUpdater", "Failed to delete slash command: {:?}", e);
                    errors.push(
                        RuntimeError::new(e.to_string())
                            .with_target("delete_global_command")
                            .with_context(format!("Failed to delete slash command '{}'", command.name))
                    );
                } else {
                    info!(target: "InteractionUpdater", "Deleted slash command '{}'", command.name);
                    success.push(format!("delete_global_command::{}", command.name));
                }
            }
        }
    }
    // then, delete all commands for the administrator guild
    {
        let guild_id = crate::constants::ADMIN_GUILD.into();

        match http.get_guild_commands(&application.id, &guild_id).await {
            Ok(Ok(commands)) => {
                for command in commands {
                    match http.delete_guild_command(&application.id, &guild_id, &command.id).await {
                        Ok(_) => {
                            info!(target: "InteractionUpdater", "Deleted local admin slash command '{}'", command.name);
                            success.push(format!("delete_admin_guild_command::{}", command.name));
                        }
                        Err(e) => {
                            error!(target: "InteractionUpdater", "Failed to delete slash command: {:?}", e);
                            errors.push(
                                RuntimeError::new(e.to_string())
                                    .with_target("delete_admin_guild_command")
                                    .with_context(format!("Failed to delete local admin slash command '{}'", command.name))
                            );
                        }
                    }
                }
            },
            Ok(Err(e)) => {
                error!(target: "InteractionUpdater", "Failed to fetch all slash commands from the admin guild: {e:#?}");
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("fetch_guild_commands")
                        .with_context(format!("Failed to fetch all slash commands from the admin guild: {e:#?}"))
                );
            }
            Err(e) => {
                error!(target: "InteractionUpdater", "Failed to fetch all slash commands from the admin guild: {e:#?}");
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("fetch_guild_commands")
                        .with_context(format!("Failed to fetch all slash commands from the admin guild: {e:#?}"))
                );
            }
        }
    }

    // 'finally', create all global commands

    // ping
    {
        let ping = ping_slash();
        let r = http.create_global_application_command(&application.id, ping).await;
        if let Err(e) = r {
            error!(target: "InteractionUpdater", "Failed to create slash command 'ping': {:?}", e);
            errors.push(
                RuntimeError::new(e.to_string())
                    .with_target("create_global_application_command")
                    .with_context("Failed to create slash command 'ping'")
            );
        } else {
            info!(target: "InteractionUpdater", "Created slash command 'ping'");
            success.push("global_command::ping".to_string());
        }
    }

    // citation
    {
        let citation = citation_slash();
        let r = http.create_global_application_command(&application.id, citation).await;
        if let Err(e) = r {
            error!(target: "InteractionUpdater", "Failed to create slash command 'citation': {:?}", e);
            errors.push(
                RuntimeError::new(e.to_string())
                    .with_target("create_global_application_command")
                    .with_context("Failed to create slash command 'citation'")
            );
        } else {
            info!(target: "InteractionUpdater", "Created slash command 'citation'");
            success.push("global_command::citation".to_string());
        }
    }

    // guild_rank
    {
        let guild_rank = guild_rank_slash();
        let r = http.create_global_application_command(&application.id, guild_rank).await;
        if let Err(e) = r {
            error!(target: "InteractionUpdater", "Failed to create slash command 'guild_rank': {:?}", e);
            errors.push(
                RuntimeError::new(e.to_string())
                    .with_target("create_global_application_command")
                    .with_context("Failed to create slash command 'guild_rank'")
            );
        } else {
            info!(target: "InteractionUpdater", "Created slash command 'guild_rank'");
            success.push("global_command::guild_rank".to_string());
        }
    }

    //top
    {
        let top = top_slash();
        let r = http.create_global_application_command(&application.id, top).await;
        match r {
            Err(e) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'top': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'top'")
                );
            }
            Ok(Err(e)) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'top': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'top'")
                );
            }
            Ok(Ok(_)) => {
                info!(target: "InteractionUpdater", "Created slash command 'top'");
                success.push("global_command::top".to_string());
            }
        }
    }

    // cookies
    {
        let cookies = cookies_slash();
        let r = http.create_global_application_command(&application.id, cookies).await;
        match r {
            Err(e) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'cookies': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'cookies'")
                );
            }
            Ok(Err(e)) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'cookies': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'cookies'")
                );
            }
            Ok(Ok(_)) => {
                info!(target: "InteractionUpdater", "Created slash command 'cookies'");
                success.push("global_command::cookies".to_string());
            }
        }
    }

    // avatar
    {
        let avatar = avatar_slash();
        let r = http.create_global_application_command(&application.id, avatar).await;
        match r {
            Err(e) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'avatar': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'avatar'")
                );
            }
            Ok(Err(e)) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'avatar': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'avatar'")
                );
            }
            Ok(Ok(_)) => {
                info!(target: "InteractionUpdater", "Created slash command 'avatar'");
                success.push("global_command::avatar".to_string());
            }
        }
    }

    // banner
    {
        let banner = banner_slash();
        let r = http.create_global_application_command(&application.id, banner).await;
        match r {
            Err(e) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'banner': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'banner'")
                );
            }
            Ok(Err(e)) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'banner': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'banner'")
                );
            }
            Ok(Ok(_)) => {
                info!(target: "InteractionUpdater", "Created slash command 'banner'");
                success.push("global_command::banner".to_string());
            }
        }
    }


    // rateit
    {
        let rateit = rateit_slash();
        let r = http.create_global_application_command(&application.id, rateit).await;
        match r {
            Err(e) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'rateit': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'rateit'")
                );
            }
            Ok(Err(e)) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'rateit': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'rateit'")
                );
            }
            Ok(Ok(_)) => {
                info!(target: "InteractionUpdater", "Created slash command 'rateit'");
                success.push("global_command::rateit".to_string());
            }
        }
    }

    // unacceptable
    {
        let unacceptable = unacceptable_slash();
        let r = http.create_global_application_command(&application.id, unacceptable).await;
        match r {
            Err(e) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'unacceptable': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'unacceptable'")
                );
            }
            Ok(Err(e)) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'unacceptable': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'unacceptable'")
                );
            }
            Ok(Ok(_)) => {
                info!(target: "InteractionUpdater", "Created slash command 'unacceptable'");
                success.push("global_command::unacceptable".to_string());
            }
        }
    }

    // welcome
    {
        let welcome = welcome_slash();
        let r = http.create_global_application_command(&application.id, welcome).await;
        match r {
            Err(e) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'welcome': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'welcome'")
                );
            }
            Ok(Err(e)) => {
                error!(target: "InteractionUpdater", "Failed to create slash command 'welcome': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command 'welcome'")
                );
            }
            Ok(Ok(_)) => {
                info!(target: "InteractionUpdater", "Created slash command 'welcome'");
                success.push("global_command::welcome".to_string());
            }
        }
    }

    // 8ball
    {
        let eight_ball = eight_ball_slash();
        let r = http.create_global_application_command(&application.id, eight_ball).await;
        match r {
            Err(e) => {
                error!(target: "InteractionUpdater", "Failed to create slash command '8ball': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command '8ball'")
                );
            }
            Ok(Err(e)) => {
                error!(target: "InteractionUpdater", "Failed to create slash command '8ball': {:?}", e);
                errors.push(
                    RuntimeError::new(e.to_string())
                        .with_target("create_global_application_command")
                        .with_context("Failed to create slash command '8ball'")
                );
            }
            Ok(Ok(_)) => {
                info!(target: "InteractionUpdater", "Created slash command '8ball'");
                success.push("global_command::8ball".to_string());
            }
        }
    }



    // admin_reload_commands [local]
    {
        let admin_reload_commands = admin_reload_commands_slash();
        let r = http.create_guild_application_command(
            &application.id,
            &crate::constants::ADMIN_GUILD.into(),
            admin_reload_commands
        ).await;
        if let Err(e) = r {
            error!(target: "InteractionUpdater", "Failed to create slash command 'admin_reload_commands': {:?}", e);
            errors.push(
                RuntimeError::new(e.to_string())
                    .with_target("create_local_application_command")
                    .with_context("Failed to create slash command 'admin_reload_commands'")
            );
        } else {
            info!(target: "InteractionUpdater", "Created slash command 'admin_reload_commands'");
            success.push("local_command::admin_reload_commands".to_string());
        }
    };

    // admin_reload_requests [local]
    {
        let admin_reload_requests = admin_reload_requests_slash();
        let r = http.create_guild_application_command(
            &application.id,
            &crate::constants::ADMIN_GUILD.into(),
            admin_reload_requests
        ).await;
        if let Err(e) = r {
            error!(target: "InteractionUpdater", "Failed to create slash command 'admin_reload_requests': {:?}", e);
            errors.push(
                RuntimeError::new(e.to_string())
                    .with_target("create_local_application_command")
                    .with_context("Failed to create slash command 'admin_reload_requests'")
            );
        } else {
            info!(target: "InteractionUpdater", "Created slash command 'admin_reload_requests'");
            success.push("local_command::admin_reload_requests".to_string());
        }
    };

    // admin_reload_langs [local]
    {
        let admin_reload_langs = admin_reload_langs_slash();
        let r = http.create_guild_application_command(
            &application.id,
            &crate::constants::ADMIN_GUILD.into(),
            admin_reload_langs
        ).await;
        if let Err(e) = r {
            error!(target: "InteractionUpdater", "Failed to create slash command 'admin_reload_langs': {:?}", e);
            errors.push(
                RuntimeError::new(e.to_string())
                    .with_target("create_local_application_command")
                    .with_context("Failed to create slash command 'admin_reload_langs'")
            );
        } else {
            info!(target: "InteractionUpdater", "Created slash command 'admin_reload_langs'");
            success.push("local_command::admin_reload_langs".to_string());
        }
    };

    // admin_memory_report [local]
    {
        let admin_reload_langs = admin_memory_report_slash();
        let r = http.create_guild_application_command(
            &application.id,
            &crate::constants::ADMIN_GUILD.into(),
            admin_reload_langs
        ).await;
        if let Err(e) = r {
            error!(target: "InteractionUpdater", "Failed to create slash command 'admin_memory_report': {:?}", e);
            errors.push(
                RuntimeError::new(e.to_string())
                    .with_target("create_local_application_command")
                    .with_context("Failed to create slash command 'admin_memory_report'")
            );
        } else {
            info!(target: "InteractionUpdater", "Created slash command 'admin_memory_report'");
            success.push("local_command::admin_reload_langs".to_string());
        }
    };

    // return the result
    (
        if success.is_empty() { None } else { Some(success) },
        if errors.is_empty() { None } else { Some(errors) }
    )
}



//
//
// commands builders
//
//



fn ping_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "ping",
        "🏓 Get Kady's latency",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "ping", "🏓 Obtenez la latence de Kady")
}



fn guild_rank_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "guild_rank",
        "🏆 Get the guild rank of a player",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "serveur_xp", "🏆 Obtenez le rang du joueur sur le serveur")
        .set_dm_permission(false)
        .add_option(
            ApplicationCommandOption::new(ApplicationCommandOptionType::User, "user", "The user you want to get the rank of", false)
                .add_description_localization("fr", "L'utilisateur dont vous voulez obtenir le rang")
        )
}

fn top_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "top",
        "🏆 Get the rankings for the server... or for the whole world!",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "top", "🏆 Obtenez les classements du serveur... ou du monde entier !")
        .set_dm_permission(true)
        .add_option(
            ApplicationCommandOption::new(
                ApplicationCommandOptionType::SubCommand,
                "xp",
                "⭐ The xp rankings of the server",
                false
            ).add_description_localization("fr", "⭐ Le classement de l'expérience du serveur")
        )
        .add_option(
            ApplicationCommandOption::new(
                ApplicationCommandOptionType::SubCommand,
                "cookies", // reputation
                "🍪 Who have the most cookies ??",
                false
            ).add_description_localization("fr", "🍪 Qui as le plus de cookies ??")
        )
}



fn admin_reload_commands_slash() -> ApplicationCommand {
    ApplicationCommand::new_local(
        "admin_reload_commands",
        "🔄 Update all slash commands",
        ApplicationCommandType::ChatInput,
        crate::constants::ADMIN_GUILD.into(),
    ).add_localization("fr", "admin_reload_commands", "🔄 Mettre à jour toutes les commandes slash")
}

fn admin_reload_requests_slash() -> ApplicationCommand {
    ApplicationCommand::new_local(
        "admin_reload_requests",
        "🔄 Update the request list for the database",
        ApplicationCommandType::ChatInput,
        crate::constants::ADMIN_GUILD.into(),
    ).add_localization("fr", "admin_reload_requests", "🔄 Mettre à jour la liste des requêtes pour la base de données")
}

fn admin_reload_langs_slash() -> ApplicationCommand {
    ApplicationCommand::new_local(
        "admin_reload_langs",
        "🔄 Update the translations",
        ApplicationCommandType::ChatInput,
        crate::constants::ADMIN_GUILD.into(),
    ).add_localization("fr", "admin_reload_langs", "🔄 Mettre à jour les traductions")
}

fn admin_memory_report_slash() -> ApplicationCommand {
    ApplicationCommand::new_local(
        "admin_memory_report",
        "📄 Get the memory usage",
        ApplicationCommandType::ChatInput,
        crate::constants::ADMIN_GUILD.into(),
    ).add_localization("fr", "admin_memory_report", "📄 Obtenir l'utilisation de la mémoire")
}



fn citation_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "citation",
        "✉️ Send a beautiful citation to the whole server",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "citation", "✉️ Envoyez une magnifique citation à tout le serveur")
        .set_dm_permission(false)
        .add_option(
            ApplicationCommandOption::new(ApplicationCommandOptionType::String, "citation", "Your citation here", true)
                .add_description_localization("fr", "Votre citation")
        )
}

fn avatar_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "avatar",
        "👤 Get the beautiful avatar of yourself or your friends",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "avatar", "👤 Obtenez le magnifique avatar de vous-même ou de vos amis")
        .set_dm_permission(true)
        .add_option(
            ApplicationCommandOption::new(ApplicationCommandOptionType::User, "user", "The user (optional)", false)
                .add_name_localization("fr", "utilisateur")
                .add_description_localization("fr", "L'utilisateur (optionnel)")
        )
}

fn banner_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "banner",
        "👤 Get the beautiful banner of yourself or your friends",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "avatar", "👤 Obtenez la magnifique bannière de vous-même ou de vos amis")
        .set_dm_permission(true)
        .add_option(
            ApplicationCommandOption::new(ApplicationCommandOptionType::User, "user", "The user (optional)", false)
                .add_name_localization("fr", "utilisateur")
                .add_description_localization("fr", "L'utilisateur (optionnel)")
        )
}

fn cookies_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "cookies",
        "🍪 A cookie ?",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "cookies", "🍪 Un cookie ?")
        .set_dm_permission(true)
        .add_option(
            ApplicationCommandOption::new(
                ApplicationCommandOptionType::SubCommand,
                "daily",
                "🍪 Get your daily cookie by solving an enigma",
                false
            )
                .add_name_localization("fr", "journalier")
                .add_description_localization("fr", "🍪 Obtient ton cookie quotidien en résolvant une énigme")
        )
        .add_option(
        ApplicationCommandOption::new(
            ApplicationCommandOptionType::SubCommand,
            "donate",
            "🍪 Give one or more cookies to your friends !",
            false
        )
            .add_name_localization("fr", "donner")
            .add_description_localization("fr", "🍪 Donne un ou plusieurs cookies à tes amis !")
            .add_option(
                ApplicationCommandOption::new(
                    ApplicationCommandOptionType::User,
                    "user",
                    "👤 The lucky person who will receive your cookie(s)",
                    true
                )
                    .add_name_localization("fr", "utilisateur")
                    .add_description_localization("fr", "👤 Le chanceux qui va recevoir votre/vos cookie(s)")
            )
            .add_option(
                ApplicationCommandOption::new(
                    ApplicationCommandOptionType::Number,
                    "number",
                    "🍪 The number of cookies you want to donate",
                    true
                )
                    .add_name_localization("fr", "nombre")
                    .add_description_localization("fr", "🍪 Le nombre de cookies que vous souhaiter donné(e)")
            )
    )
}



fn rateit_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "rateit",
        "📒 Will you have the best note ?",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "note", "📒 Allez-vous avoir la meilleure note ?")
}



fn eight_ball_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "8ball",
        "🎱 Will the chance be with you ?",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "8ball", "🎱 La chance sera-elle de ton coté ?")
        .add_option(
            ApplicationCommandOption::new(
                ApplicationCommandOptionType::String,
                "question",
                "😁 Your question",
                true
            )
                .add_description_localization("fr", "😁 Ta question")
        )
}

fn unacceptable_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "unacceptable",
        "💥 This is definitely unacceptable!",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "inacceptable", "💥 C'est définitivement inacceptable !")
}

fn welcome_slash() -> ApplicationCommand {
    ApplicationCommand::new_global(
        "welcome",
        "👋 Welcome the new members",
        ApplicationCommandType::ChatInput
    ).add_localization("fr", "bienvenue", "👋 Souhaite la bienvenue")
}