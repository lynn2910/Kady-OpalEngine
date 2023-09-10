//! This module is reserved to the Discord Interactions-based scripts.
//!
//! In other words, when a user with the interactions listed below, the bot will respond to it.
//!
//! ## This include:
//! - Slash Commands
//! - Components
//! - Buttons
//! - Select Menus
//! - Context Menus
//! - Modals

use std::collections::HashMap;
use log::{error, warn};
use client::manager::events::Context;
use client::models::events::InteractionCreate;
use client::models::guild::{Guild, GuildId, GuildMember};
use client::models::message::MessageBuilder;
use client::models::user::{Application, ClientUser, User, UserId};
use translation::{Language, message};
use error::{Error, EventError, Result};
use crate::constants::DEFAULT_LANG;

mod slashs;
mod buttons;
mod modal;

/// Get the guild locale, or the default one if the guild locale is not supported.
pub(crate) fn get_guild_locale(guild_locale: &Option<String>) -> String {
    if guild_locale.is_none() {
        return DEFAULT_LANG.to_string();
    }

    let translations = if let Ok(t) = translation::TRANSLATIONS.read() {
        t
    } else {
        return DEFAULT_LANG.to_string();
    };

    let lang = Language(guild_locale.clone().unwrap());

    if translations.contains_key(&lang) {
        return guild_locale.as_ref().unwrap().to_string();
    }

    DEFAULT_LANG.to_string()
}

/// Handle the slash commands.
pub(crate) async fn slash_command_received(ctx: &Context, payload: &InteractionCreate){
    // we are now sure that the interaction is a slash command
    let command = match payload.interaction.data.as_ref() {
        Some(d) => d,
        None => {
            error!(target: "Runtime", "cannot acquire the slashs command informations from {:#?}", payload.interaction);
            return;
        }
    };

    let name = command.name.clone().unwrap_or(String::new());
    // DON'T USE "" AS A COMMAND NAME!!
    match name.as_str() {
        "ping" => slashs::ping::triggered(ctx, payload).await,
        "citation" => slashs::citation::triggered(ctx, payload).await,
        "admin_reload_commands" | "admin_update_commands" => slashs::admin::admin_reload_slashs::triggered(ctx, payload).await,
        "admin_reload_requests" => slashs::admin::admin_reload_requests::triggered(ctx, payload).await,
        "admin_reload_langs" => slashs::admin::admin_reload_langs::triggered(ctx, payload).await,
        "admin_memory_report" => slashs::admin::admin_memory_report::triggered(ctx, payload).await,
        "guild_rank" => slashs::xp::guild_rank::triggered(ctx, payload).await,
        "top" => slashs::top::triggered(ctx, payload).await,
        "cookies" => slashs::cookies::triggered(ctx, payload).await,
        _ => {
            let local = get_guild_locale(&payload.interaction.guild_locale);
            if let Err(e) = payload.interaction.reply(&ctx.skynet, unknown_command(local)).await {
                warn!(target: "EventHandler", "Failed to reply to slash command: {:?}", e);
            };
        }
    }
}

pub(crate) async fn button_received(ctx: &Context, payload: &InteractionCreate){
    let button = payload.interaction.data.as_ref().unwrap();

    let (custom_id, query) = {
        let custom_id = button.custom_id.clone().unwrap_or(String::new());
        let mut split = custom_id.splitn(2, '&');

        (
            split.next().unwrap_or(Default::default()).to_string(),
            split.next().unwrap_or(Default::default()).to_string(),
        )
    };

    let query_string = match querify(query) {
        Ok(q) => q,
        Err(e) => {
            error!(target: "InteractionHandler", "The runtime received an error while querying the button custom_id: {e:#?}");
            return;
        }
    };

    match custom_id.as_str() {
        "CAPTCHA_REQUEST" => buttons::captcha_request::triggered(ctx, payload, query_string).await,
        "CAPTCHA_TRY" => buttons::captcha_try::triggered(ctx, payload, query_string).await,
        "ANSWER_COOKIES_QUIZ" => buttons::cookies::triggered(ctx, payload).await,
        _ => {
            if payload.interaction.channel_id.is_none() { return; }
            let _ = payload.interaction.reply(
                &ctx.skynet,
                unknown_button(get_guild_locale(&payload.interaction.guild_locale))
            ).await;
        }
    }
}

pub(crate) async fn modal_received(ctx: &Context, payload: &InteractionCreate){
    let modal = payload.interaction.data.as_ref().unwrap();

    let (custom_id, query) = {
        let custom_id = modal.custom_id.clone().unwrap_or(String::new());
        let mut split = custom_id.splitn(2, '&');

        (
            split.next().unwrap_or(Default::default()).to_string(),
            split.next().unwrap_or(Default::default()).to_string(),
        )
    };

    #[allow(unused_variables)]
    let query_string = match querify(query) {
        Ok(q) => q,
        Err(e) => {
            error!(target: "InteractionHandler", "The runtime received an error while querying the modal custom_id: {e:#?}");
            return;
        }
    };

    match custom_id.as_str() {
        "COOKIE_USER_QUIZ_ANSWER" => modal::cookie_quiz_answer::triggered(ctx, payload).await,
        _ => {
            if payload.interaction.channel_id.is_none() { return; }
            let _ = payload.interaction.reply(
                &ctx.skynet,
                unknown_modal(get_guild_locale(&payload.interaction.guild_locale))
            ).await;
        }
    }
}

/// Return a MessageBuilder with a message for an unknown command
fn unknown_command(local: String) -> MessageBuilder {
    MessageBuilder::new().set_content(message!(local, "errors::unknown_slash_command")).set_ephemeral(true)
}

/// Return a MessageBuilder for an unknown component
fn unknown_button(local: String) -> MessageBuilder {
    MessageBuilder::new().set_content(message!(local, "errors::unknown_button")).set_ephemeral(true)
}


fn unknown_modal(local: String) -> MessageBuilder {
    MessageBuilder::new().set_content(message!(local, "errors::unknown_modal")).set_ephemeral(true)
}


// UTILITY FUNCTIONS
/// Get a user from the cache or Api
pub async fn get_user(ctx: &Context, user: &UserId) -> Option<User> {
    {
        let cache = ctx.cache.read().await;

        if let Some(user) = cache.get_user(user) { return Some(user.clone()) }
    }

    match ctx.skynet.fetch_user(user).await {
        Ok(Ok(user)) => {
            let mut cache = ctx.cache.write().await;
            cache.update_user(&user);

            Some(user)
        }
        _ => None
    }
}

/// Get a user from the cache or Api
pub async fn get_guild(ctx: &Context, guild: &GuildId) -> Option<Guild> {
    {
        let cache = ctx.cache.read().await;

        if let Some(guild) = cache.get_guild(guild) { return Some(guild.clone()) }
    }

    match ctx.skynet.fetch_guild(guild).await {
        Ok(Ok(guild)) => {
            let mut cache = ctx.cache.write().await;
            cache.update_guild(&guild);

            Some(guild)
        }
        _ => None
    }
}

pub fn get_user_id(user: &Option<User>, member: &Option<GuildMember>) -> Option<UserId> {
    if let Some(u) = user { return Some(u.id.clone() ) }
    if let Some(m) = member {
        if let Some(u) = &m.user {
            return Some(u.id.clone())
        }
    }
    None
}

#[allow(dead_code)]
pub async fn get_application(ctx: &Context) -> Option<Application> {
    {
        let cache = ctx.cache.read().await;

        if let Some(app) = cache.get_application() { return Some(app.clone()) }
    }

    match ctx.skynet.fetch_application().await {
        Ok(Ok(app)) => {
            let mut cache = ctx.cache.write().await;
            cache.update_application(&app);

            Some(app)
        }
        _ => None
    }
}

pub async fn get_client_user(ctx: &Context) -> Option<ClientUser> {
    {
        let cache = ctx.cache.read().await;

        if let Some(client_user) = cache.get_client_user() { return Some(client_user.clone()) }
    }

    match ctx.skynet.fetch_client_user().await {
        Ok(Ok(client_user)) => {
            let mut cache = ctx.cache.write().await;
            cache.update_client_user(&client_user);

            Some(client_user)
        }
        _ => None
    }
}


// QUERY STRING SYSTEM


pub type QueryParams = HashMap<String, String>;

/// Parses a given query string back into a vector of key-value pairs.
pub fn querify(string: String) -> Result<QueryParams> {
    if !string.contains('&') {
        return Ok(HashMap::new())
    }

    let mut map = HashMap::new();
    for pair in string.split('&') {
        let mut it = pair.split('=');

        let kv = match (it.next(), it.next()) {
            (Some(k), Some(v)) => (k.to_string(), v.to_string()),
            (a, b) => return Err(Error::Event(EventError::Runtime(format!("Invalid querystring was received: ({a:?}, {b:?})")))),
        };
        map.insert(kv.0, kv.1);
    }
    Ok(map)
}