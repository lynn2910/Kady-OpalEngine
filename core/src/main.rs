mod constants;
mod events;
mod tasks;
mod crates;
mod interaction_constructor;
mod scripts;
mod assets;
mod database_cleaner;

use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use log::{error, info, warn};
use tokio::sync::RwLock;
use api::{Api, ApiState, SecurityContainer};
use archive::Archive;
use client::Client;
use client::manager::events::{Context, EventHandler};
use client::manager::http::HttpConfiguration;
use client::models::components::message_components::ComponentType;
use client::models::events::{GuildCreate, GuildDelete, GuildMemberUpdate, InteractionCreate, MessageCreate, Ready};
use client::models::interaction::InteractionType;
use client::models::message::MessageBuilder;
use config::Config;
use database::Database;
use translation::message;
use clap::Parser;
use crate::scripts::get_guild;

extern crate translation;

struct Handler;

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(target: "client", "Shard {} ready", ready.shard);

        let application = {
            let cache = ctx.cache.read().await;
            cache.get_application().cloned()
        }.unwrap();

        info!("Connected as {}", application.name);
    }

    async fn guild_create(&self, ctx: Context, payload: GuildCreate) {
        // add guild to cache and lazy load channels
        if let Some(guild) = payload.guild.as_ref() {
            let mut cache = ctx.cache.write().await;

            cache.update_guild(guild);

            let channels = match ctx.skynet.fetch_guild_channels(&guild.id).await {
                Ok(chl) => chl,
                Err(e) => {
                    error!(target: "Runtime", "An error occured while fetching channels from the guild '{}': {e:#?}", guild.id);
                    return;
                }
            };
            match channels {
                Ok(channels) => {
                    for channel in channels {
                        cache.update_channel(&channel);
                    }
                },
                Err(e) => {
                    warn!(target: "EventHandler", "Failed to fetch channels for guild {}: {:?}", guild.id, e);
                }
            }
        }

        events::guild_add_remove::guild_create(&ctx, payload).await;
    }

    async fn guild_delete(&self, ctx: Context, payload: GuildDelete) {
        if payload.unavailable { return; } // The guild is unavailable, but the client is in the guild

        // remove the guild from the cache
        let guild = {
            let mut cache = ctx.cache.write().await;
            cache.delete_guild(payload.id.clone())
        };

        events::guild_add_remove::guild_remove(&ctx, payload, guild).await;
    }

    async fn message_create(&self, ctx: Context, payload: MessageCreate) {
        // ensure the guild exists in the database
        if let Some(guild_id) = payload.guild_id.clone()         {
            if let Some(db) = ctx.data.read().await.get::<Database>() {
                let pool = db.get_pool().await;
                let requests = db.get_requests().await;

                let id = guild_id.to_string();

                let has_res = sqlx::query(requests.guilds.has.as_str())
                    .bind(&id)
                    .fetch_one(pool.deref())
                    .await;

                // if 'has_res' is an Err(_), well, the data don't exist :)
                if has_res.is_err() {
                    let _ = sqlx::query(requests.guilds.ensure.as_str())
                        .bind(&id)
                        .execute(pool.deref())
                        .await;

                    let guild = get_guild(&ctx, &guild_id).await;

                    if let Some(g) = guild {
                        if let Some(config) = ctx.get_data().await {
                            events::guild_add_remove::send_new_guild_message(
                                &ctx,
                                &g,
                                &config
                            ).await;
                        } else {
                            warn!(target: "Runtime", "The config object was not found in the context data");
                        }
                    }
                }
            }
        }

        // add message to cache
        {
            let mut cache = ctx.cache.write().await;
            cache.update_message(&payload.message.channel_id, payload.message.clone());
        }

        events::message_create::triggered(ctx, payload).await;
    }

    async fn message_delete(&self, ctx: Context, payload: client::models::events::MessageDelete) {
        // ensure the guild exists in the database
        if let Some(guild_id) = payload.guild_id.clone()         {
            if let Some(db) = ctx.data.read().await.get::<Database>() {
                let pool = db.get_pool().await;
                let requests = db.get_requests().await;

                let id = guild_id.to_string();

                let has_res = sqlx::query(requests.guilds.has.as_str())
                    .bind(&id)
                    .fetch_one(pool.deref())
                    .await;

                // if 'has_res' is an Err(_), well, the data don't exist :)
                if has_res.is_err() {
                    let _ = sqlx::query(requests.guilds.ensure.as_str())
                        .bind(&id)
                        .execute(pool.deref())
                        .await;

                    let guild = get_guild(&ctx, &guild_id).await;

                    if let Some(g) = guild {
                        if let Some(config) = ctx.get_data().await {
                            events::guild_add_remove::send_new_guild_message(
                                &ctx,
                                &g,
                                &config
                            ).await;
                        } else {
                            warn!(target: "Runtime", "The config object was not found in the context data");
                        }
                    }
                }
            }
        }

        let (channel, id) = (payload.channel_id.clone(), payload.id.clone());

        events::message_delete::triggered(&ctx, payload).await;

        // remove message from cache
        {
            let mut cache = ctx.cache.write().await;
            cache.delete_message(&channel, &id);
        }
    }

    async fn guild_member_add(&self, ctx: Context, payload: client::models::events::GuildMemberAdd) {
        // ensure the guild exists in the database
        {
            if let Some(db) = ctx.data.read().await.get::<Database>() {
                let pool = db.get_pool().await;
                let requests = db.get_requests().await;

                let id = payload.guild_id.to_string();

                let has_res = sqlx::query(requests.guilds.has.as_str())
                    .bind(&id)
                    .fetch_one(pool.deref())
                    .await;

                // if 'has_res' is an Err(_), well, the data don't exist :)
                if has_res.is_err() {
                    let _ = sqlx::query(requests.guilds.ensure.as_str())
                        .bind(&id)
                        .execute(pool.deref())
                        .await;

                    let guild = get_guild(&ctx, &payload.guild_id).await;

                    if let Some(g) = guild {
                        if let Some(config) = ctx.get_data().await {
                            events::guild_add_remove::send_new_guild_message(
                                &ctx,
                                &g,
                                &config
                            ).await;
                        } else {
                            warn!(target: "Runtime", "The config object was not found in the context data");
                        }
                    }
                }
            }
        }

        // add to the cache
        {
            let mut cache = ctx.cache.write().await;
            if let Some(user) = &payload.member.user {
                cache.update_guild_member(&payload.guild_id, &user.id, &payload.member);
            } else {
                #[cfg(feature = "debug")]
                warn!(target: "EventHandler", "Member {:?} has no user", payload.member.user.clone().map(|u| u.id));
            }
        }

        events::guild_member_add::triggered(&ctx, payload).await;
    }

    async fn guild_member_update(&self, ctx: Context, payload: GuildMemberUpdate) {
        if let Some(guild_id) = &payload.member.guild_id {
            let mut cache = ctx.cache.write().await;

            if let Some(user) = &payload.member.user {
                cache.update_guild_member(
                    guild_id,
                    &user.id,
                    &payload.member
                );
            }
        }
    }

    #[allow(unused)]
    async fn start(&self, ctx: Context) {
        info!(target: "client", "Client started");

        // let _ = interaction_constructor::instance_trigger(ctx.skynet.clone(), ctx.cache.clone()).await;

        // REMOVE ABSOLUTELY
        // {
        //     let _ = ctx.skynet.send_message(
        //         &client::models::channel::ChannelId(client::models::Snowflake("1124011642494132264".into())),
        //         MessageBuilder::new()
        //             .set_content("> 🔒 ** ** **Cliquez ci-dessous pour passer le captcha de vérification.**")
        //             .add_component(
        //                 client::models::components::components::Component::ActionRow(
        //                     client::models::components::components::ActionRow::new()
        //                         .add_component(client::models::components::components::Component::Button(
        //                             client::models::components::components::Button::new("CAPTCHA_REQUEST")
        //                                 .set_emoji(client::models::components::Emoji::new(None, "🔑"))
        //                                 .set_label("Passer la vérification")
        //                                 .set_disabled(false)
        //                                 .set_style(client::models::components::components::ButtonStyle::Secondary)
        //                         ))
        //                 )
        //             )
        //     ).await;
        // }
    }

    async fn interaction_create(&self, ctx: Context, payload: InteractionCreate) {
        // ensure the guild exists in the database
        if let Some(guild_id) = &payload.interaction.guild_id {
            if let Some(db) = ctx.data.read().await.get::<Database>() {
                let pool = db.get_pool().await;
                let requests = db.get_requests().await;

                let id = guild_id.to_string();

                let has_res = sqlx::query(requests.guilds.has.as_str())
                    .bind(&id)
                    .fetch_one(pool.deref())
                    .await;

                // if 'has_res' is an Err(_), well, the data don't exist :)
                if has_res.is_err() {
                    let _ = sqlx::query(requests.guilds.ensure.as_str())
                        .bind(&id)
                        .execute(pool.deref())
                        .await;

                    let guild = get_guild(&ctx, guild_id).await;

                    if let Some(g) = guild {
                        if let Some(config) = ctx.get_data().await {
                            events::guild_add_remove::send_new_guild_message(
                                &ctx,
                                &g,
                                &config
                            ).await;
                        } else {
                            warn!(target: "Runtime", "The config object was not found in the context data");
                        }
                    }
                }
            }
        }

        // we skip any bot
        if let Some(user) = &payload.interaction.user {
            if user.bot.unwrap_or(false) {
                return;
            }
        }

        match payload.interaction.interaction_type {
            InteractionType::Ping => {
                println!("Interaction ping, tf ?");
                return;
            },
            InteractionType::ApplicationCommand => {
                // send to the handler
                scripts::slash_command_received(
                    &ctx,
                    &payload
                ).await;
            },
            InteractionType::MessageComponent => {
                if payload.interaction.data.is_none() { return; }

                match payload.interaction.data.clone().unwrap().component_type {
                    Some(ComponentType::Button) => scripts::button_received(&ctx, &payload).await,
                    Some(_) => {
                        // we notify that we don't understand what the fuck is this interaction

                        let msg = MessageBuilder::new().set_content(
                            message!(scripts::get_guild_locale(&payload.interaction.guild_locale), "unknown_component").to_string()
                        );

                        let _ = payload.interaction.reply(&ctx.skynet, msg).await;
                    },
                    _ => {}
                }
            },
            InteractionType::ModalSubmit => {
                scripts::modal_received(&ctx, &payload).await;
            },
            _ => {
                println!("Interaction type {:?}", payload.interaction.interaction_type);
            }
        }

        // update cache for user, member & channel :)
        if let Some(channel) = &payload.interaction.channel {
            let mut cache = ctx.cache.write().await;
            cache.update_channel(channel)
        }

        if let Some(guild_id) = &payload.interaction.guild_id {
            if let Some(member) = &payload.interaction.member {
                if let Some(user) = &member.user {
                    let mut cache = ctx.cache.write().await;
                    cache.update_guild_member(guild_id, &user.id, member);
                    cache.update_user(user);
                } else if let Some(user) = &payload.interaction.user {
                    let mut cache = ctx.cache.write().await;
                    cache.update_user(user)
                }
            }
        }
    }
}

#[allow(unused_mut)]
async fn start(cli_args: CliArgs) {
    // init env logger
    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Info)
        .filter(Some("sqlx"), log::LevelFilter::Warn)
        .init();

    // load config
    let config: Config = config::load_from(constants::CONFIG_PATH.into()).unwrap();

    // write the PID in a file
    {
        let pid = std::process::id();
        std::fs::write(config.pid.clone(), pid.to_string()).expect("Cannot write PID in a file, some real shit is happening");
    }

    // load archive
    let mut archive = Archive::open(
        PathBuf::from_str(config.security.archive_path.as_str())
            .expect("Cannot create PathBuf for archive")
    ).expect("Failed to load archive");

    archive.save().unwrap();

    translation::init!(config.langs.clone().as_str());

    // load database
    let database: Database = match Database::connect(&archive, &config).await {
        Ok(d) => {
            database_cleaner::database_cleaner(d.clone());
            d
        },
        Err(err) => panic!("{:?}", err)
    };

    // init client
    let mut client = {
        let token = archive.get::<String>("token").expect("Failed to get token from archive");
        Client::new(
            token,
            HttpConfiguration { retry_limit: config.api.retry_limit, connect_timeout: std::time::Duration::from_secs(config.api.close_timeout) }
        ).await
    };

    // IMPORTANT
    // This is a function that will manage to stop the bot when the SIGINT or SIGTERM signals are received
    #[cfg(unix)]
    tasks::spawn_manager(client.http_manager.clone(), client.shard_manager.clone());

    let config = Arc::new(RwLock::new(config));

    // init status manager
    let status_manager = crates::status::ShardStatusManager::new(
        client.shard_manager.clone(),
        config.clone(),
        client.cache.clone()
    ).await;

    // Add useful informations & managers to the client data
    {
        let mut data = client.data.write().await;
        // add the config
        data.insert::<Config>(config.read().await.clone());
        // add the database
        data.insert::<Database>(database.clone());
        // add the status manager
        data.insert::<crates::status::ShardStatusManager>(status_manager);
        // add the captcha container
        data.insert::<features::captcha::CaptchaContainer>(features::captcha::CaptchaContainer::new());
        // add the xp cooldown container
        data.insert::<features::xp::XpCooldownContainer>(features::xp::XpCooldownContainer::new());
        // add the font container
        let font_container = features::xp::image_gen::FontContainer::new()
            .expect("Cannot load the fonts for the FontContainer, What the fuck?");
        data.insert::<features::xp::image_gen::FontContainer>(font_container);

    }

    // start the nugget updater
    crates::cookies::nuggets::nugget_updater_task(
        database.clone(),
        client.http_manager.client.clone(),
        client.cache.clone()
    );

    // register events
    client.event_handler(Handler);

    let shard_manager_clone = client.shard_manager.clone();
    let http_manager_clone = client.http_manager.clone();

    // Starting the API
    {
        let mut informations = Api::new(
            Arc::new(ApiState::new(
                client.cache.clone(),
                SecurityContainer::new(
                    client.shard_manager.clone(),
                    database
                ),
                Arc::new(RwLock::new(
                    config.read().await.api.declared_files.clone()
                ))
            ))
        );
        api::start(&mut informations, format!("{}:{}", cli_args.domain, cli_args.port).as_str());

        let mut data = client.data.write().await;
        data.insert::<Api>(informations);
    }

    // start the client with the intents
    // this function will block the thread until the client is stopped
    let intents = config.read().await.intents;
    client.start(intents).await.expect("Failed to run client");

    tasks::stop(http_manager_clone, shard_manager_clone).await;
}


#[tokio::main]
async fn main(){
    let args = CliArgs::parse();

    start(args).await;
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// The domain that the API will listen to
    #[arg(short, long)]
    domain: String,

    /// The port of which the API will be listening to
    #[arg(short, long)]
    port: u16
}