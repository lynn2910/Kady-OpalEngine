use std::collections::HashMap;
use std::sync::Arc;
use lazy_static::lazy_static;
use log::{error, info};
use tokio::sync::RwLock;
use client::manager::cache::CacheManager;
use client::manager::http::Http;
use client::models::guild::GuildId;
use client::models::interaction::{ApplicationCommand, ApplicationCommandOption, ApplicationCommandOptionType, ApplicationCommandType};
use client::models::user::Application;
use error::RuntimeError;
use crate::constants::ADMIN_GUILD;

#[allow(dead_code)]

/// This function will send a post request for all scripts
///
///  Don't call too much or you will get rate-limited very badly
pub(crate) async fn instance_trigger(
    http: Arc<Http>,
    cache: Arc<RwLock<CacheManager>>
) -> (Option<Vec<String>>, Option<Vec<RuntimeError>>)
{
    let application = {
        let app = {
            let cache = cache.read().await;

            cache.get_application().cloned()
        };

        if app.is_none() {
            match http.fetch_application().await {
                Ok(Ok(app)) => {
                    let mut cache = cache.write().await;
                    cache.update_application(&app);

                    app
                }
                Ok(Err(e)) => return (
                    None,
                    Some(
                        vec![
                            RuntimeError::new(e.to_string())
                                .with_context("Failed to fetch application")
                                .with_target("fetching_application")
                        ]
                    )
                ),
                Err(e) => return (
                    None,
                    Some(
                        vec![
                            RuntimeError::new(e.to_string())
                                .with_context("Failed to fetch application")
                                .with_target("fetching_application")
                        ]
                    )
                )
            }
        } else {
            app.unwrap()
        }
    };

    // delete all commands
    if let Err(e) = delete_all_global_commands(http.as_ref(), &application).await {
        error!(target: "GlobalCommandSuppressor", "Cannot delete all global commands: {e:#?}");
        return (
            None,
            Some(vec![e])
        )
    };
    info!(target: "CommandsManager", "All global commands have been deleted");
    if let Err(e) = delete_guild_commands(http.as_ref(), &application, ADMIN_GUILD).await {
        error!(target: "GlobalCommandSuppressor", "Cannot delete all commands in the admin's guild: {e:#?}");
        return (
            None,
            Some(vec![e])
        )
    };
    info!(target: "CommandsManager", "All guild commands have been deleted");
    info!(target: "CommandsManager", "All application commands are gone");

    let commands = COMMANDS.read().await;
    let mut successful_operations: Vec<String> = Vec::new();
    let mut errors_occurred: Vec<RuntimeError> = Vec::new();

    for (category, commands) in commands.iter() {
        info!(target: "GlobalCommandCreator", "Creating all slash commands for {category:?}");

        for (name, command) in commands.commands.iter() {
            let res = match commands.guild.as_ref() {
                Some(g) => http.create_guild_application_command(&application.id, g, command).await,
                None => http.create_global_application_command(&application.id, command).await
            };

            match res {
                Ok(Ok(_)) => successful_operations.push(format!("Creating slash command {name:?}")),
                Ok(Err(e)) => {
                    error!(target: "GlobalCommandCreator", "An error occured while creating the command {name:?}: {e:#?}");
                    errors_occurred.push(
                        RuntimeError::new(e.message)
                            .with_target(e.code)
                    );
                }
                Err(e) => {
                    error!(target: "GlobalCommandCreator", "An error occured while trying to create the command {name:?}: {e:#?}");
                    errors_occurred.push(
                        RuntimeError::new(e.to_string())
                    );
                }
            };
        }


        info!(target: "GlobalCommandCreator", "All slash commands in the {category:?} have been created");
    }

    info!(target: "CommandsManager", "All commands had been updated");

    (
        Some(successful_operations),
        Some(errors_occurred)
    )
}

pub(crate) async fn delete_all_global_commands(
    http: &Http,
    application: &Application
) -> Result<(), RuntimeError>
{
    let all_commands = http.get_global_commands(&application.id).await;

    match all_commands {
        Ok(Ok(commands)) => {
            for command in commands {
                if let Err(e) = http.delete_global_command(&application.id, &command.id).await {
                    error!(target: "GlobalCommandSuppressor", "Cannot delete a global command: {e:#?}");
                };
            }

            Ok(())
        }
        Ok(Err(e)) => Err(RuntimeError::new(e.to_string())
            .with_context("Failed to fetch application")
            .with_target("fetching_application")),
        Err(e) => Err(RuntimeError::new(e.to_string())
            .with_context("Failed to send request for fetching application")
            .with_target("fetching_application"))
    }
}

pub(crate) async fn delete_guild_commands(
    http: &Http,
    application: &Application,
    guild_id: impl Into<GuildId>
) -> Result<(), RuntimeError>
{
    let guild_id: GuildId = guild_id.into();
    let all_commands = http.get_guild_commands(&application.id, &guild_id).await;

    match all_commands {
        Ok(Ok(commands)) => {
            for command in commands {
                if let Err(e) = http.delete_global_command(&application.id, &command.id).await {
                    error!(target: "GlobalCommandSuppressor", "Cannot delete a command in the guild {}: {e:#?}", guild_id.to_string());
                };
            }

            Ok(())
        }
        Ok(Err(e)) => Err(RuntimeError::new(e.to_string())
            .with_context("Failed to fetch application")
            .with_target("fetching_application")),
        Err(e) => Err(RuntimeError::new(e.to_string())
            .with_context("Failed to send request for fetching application")
            .with_target("fetching_application"))
    }
}

#[derive()]
pub(crate) struct CommandsContainer {
    pub visible: bool,
    pub guild: Option<GuildId>,
    pub commands: HashMap<String, ApplicationCommand>
}

fn commands_vec_to_hashmap(commands: Vec<ApplicationCommand>) -> HashMap<String, ApplicationCommand> {
    let mut hashmap = HashMap::default();

    for command in commands {
        hashmap.insert(
            command.name.clone(),
            command
        );
    }

    hashmap
}

pub(crate) type CommandsStorage = HashMap<CommandType, CommandsContainer>;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum CommandType {
    Common,
    Fun,
    Tools,
    Dev,
}


lazy_static! {
    static ref COMMANDS: Arc<RwLock<CommandsStorage>> = {
        let mut commands_storage: CommandsStorage = CommandsStorage::default();

        // insert the dev commands
        commands_storage.insert(
            CommandType::Dev,
            CommandsContainer {
                visible: false,
                guild: Some(ADMIN_GUILD.into()),
                commands: commands_vec_to_hashmap(vec![
                    admin_memory_report_slash(),
                    admin_reload_langs_slash(),
                    admin_reload_requests_slash(),
                    admin_reload_commands_slash()
                ])
            }
        );

        // insert funny commands
        commands_storage.insert(
            CommandType::Fun,
            CommandsContainer {
                visible: true,
                guild: None,
                commands: commands_vec_to_hashmap(vec![
                    eight_ball_slash(),
                    welcome_slash(),
                    unacceptable_slash(),
                    rateit_slash()
                ])
            }
        );

        // insert useful commands
        commands_storage.insert(
            CommandType::Tools,
            CommandsContainer {
                visible: true,
                guild: None,
                commands: commands_vec_to_hashmap(vec![
                    citation_slash(),
                    top_slash(),
                    cookies_slash(),
                    guild_rank_slash()
                ])
            }
        );

        // insert common commands
        commands_storage.insert(
            CommandType::Common,
            CommandsContainer {
                visible: true,
                guild: None,
                commands: commands_vec_to_hashmap(vec![
                    ping_slash(),
                    avatar_slash(),
                    banner_slash(),
                    user_info()
                ])
            }
        );

        Arc::new(RwLock::new(commands_storage))
    };
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

fn user_info() -> ApplicationCommand {
    ApplicationCommand::new_global("userinfo", "👤 Get informations about someone", ApplicationCommandType::ChatInput)
        .add_localization("fr", "user_info", "👤 Obtenir des informations sur quelqu'un")
        .add_option(
            ApplicationCommandOption::new(
                ApplicationCommandOptionType::User,
                "user",
                "👋 The user you want informations about (leave blank to get your infos)",
                false
            )
                .add_name_localization("fr", "utilisateur")
                .add_description_localization("fr", "👋 L'utilisateur dont tu souhaite voir les informations (laisser vide pour avoir vos infos)")
        )
}