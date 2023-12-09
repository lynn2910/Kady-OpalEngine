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
mod select_menu;

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
        "ping" => slashs::common::ping::triggered(ctx, payload).await,
        "citation" => slashs::citation::triggered(ctx, payload).await,
        "admin_reload_commands" | "admin_update_commands" => slashs::admin::admin_reload_slashs::triggered(ctx, payload).await,
        "admin_reload_requests" => slashs::admin::admin_reload_requests::triggered(ctx, payload).await,
        "admin_reload_langs" => slashs::admin::admin_reload_langs::triggered(ctx, payload).await,
        "admin_memory_report" => slashs::admin::admin_memory_report::triggered(ctx, payload).await,
        "guild_rank" => slashs::xp::guild_rank::triggered(ctx, payload).await,
        "top" => slashs::top::triggered(ctx, payload).await,
        "cookies" => slashs::cookies::triggered(ctx, payload).await,
        "avatar" => slashs::common::avatar_slash::triggered(ctx, payload).await,
        "banner" => slashs::common::banner_slash::triggered(ctx, payload).await,
        "welcome" => slashs::common::welcome::triggered(ctx, payload).await,
        "rateit" | "note" => slashs::fun::rateit::triggered(ctx, payload).await,
        "unacceptable" => slashs::fun::unacceptable::triggered(ctx, payload).await,
        "8ball" => slashs::fun::eight_ball::triggered(ctx, payload).await,
        "userinfo" => slashs::common::user_info::triggered(ctx, payload).await,
        "help" => slashs::common::help::triggered(ctx, payload).await,
        "kady" => slashs::common::kady::triggered(ctx, payload).await,
        _ => {
            let local = get_guild_locale(&payload.interaction.guild_locale);
            if let Err(e) = payload.interaction.reply(&ctx.skynet, unknown_command(local)).await {
                warn!(target: "EventHandler", "Failed to reply to slash command: {:?}", e);
            };
        }
    }
}

pub(crate) async fn button_received(ctx: &Context, payload: &InteractionCreate){
    if payload.interaction.data.is_none() {
        error!(target: "ButtonReceived", "No button's data was provided (wtf ?)")
    }

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
        "SUGGESTION" => buttons::kady::suggestion::triggered(ctx, payload).await,
        _ => {
            if payload.interaction.channel_id.is_none() { return; }
            let _ = payload.interaction.reply(
                &ctx.skynet,
                unknown_button(get_guild_locale(&payload.interaction.guild_locale))
            ).await;
        }
    }
}

pub(crate) async fn select_menu_received(ctx: &Context, payload: &InteractionCreate){
    let button = payload.interaction.data.as_ref().unwrap();

    let (custom_id, query) = {
        let custom_id = button.custom_id.clone().unwrap_or(String::new());
        let mut split = custom_id.splitn(2, '&');

        (
            split.next().unwrap_or_default().to_string(),
            split.next().unwrap_or_default().to_string(),
        )
    };

    let query_string = match querify(query) {
        Ok(q) => q,
        Err(e) => {
            error!(target: "InteractionHandler", "The runtime received an error while querying the select menu custom_id: {e:#?}");
            return;
        }
    };

    match custom_id.as_str() {
        "SELECT_HELP_CATEGORY" => select_menu::select_help_category::triggered(ctx, payload, query_string).await,
        _ => {
            if payload.interaction.channel_id.is_none() { return; }
            let _ = payload.interaction.reply(
                &ctx.skynet,
                unknown_select_menu(get_guild_locale(&payload.interaction.guild_locale))
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
            split.next().unwrap_or_default().to_string(),
            split.next().unwrap_or_default().to_string(),
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
        "KADY_SUGGEST_MODAL" => modal::kady::suggest::triggered(ctx, payload).await,
        "KADY_ISSUE_MODAL" => modal::kady::issue::triggered(ctx, payload).await,
        "KADY_REVIEW_MODAL" => modal::kady::review::triggered(ctx, payload).await,
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

/// Return a MessageBuilder for an unknown component
fn unknown_select_menu(local: String) -> MessageBuilder {
    MessageBuilder::new().set_content(message!(local, "errors::unknown_select_menu")).set_ephemeral(true)
}


fn unknown_modal(local: String) -> MessageBuilder {
    MessageBuilder::new().set_content(message!(local, "errors::unknown_modal")).set_ephemeral(true)
}


// UTILITY FUNCTIONS
/// Retrieves a user from user ID.
///
/// This function first checks the local cache for the user.
/// If the user is found, it is returned immediately.
///
/// If the user is not found in the local cache, the function tries to fetch the user
/// from the Skynet (possibly a remote service). If the fetch is successful,
/// the user data is updated in the local cache and then the user data is returned.
///
/// If the fetch also fails, the function returns `None`.
///
/// # Arguments
///
/// * `ctx` - A reference to the context in which this function is called.
/// This context is assumed to contain a local cache and a reference to Skynet for remote fetches.
///
/// * `user` - A reference to the `UserId` of the desired user.
///
/// # Returns
///
/// * `Option<User>` - The user data if found, `None` otherwise.
///
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

/// Retrieves a guild member from the given guild ID and user ID.
///
/// This function first checks the local cache for the member.
/// If the member is found, it is returned immediately.
///
/// If the member is not found in the local cache, the function tries to fetch the member
/// from the Skynet (possibly a remote service). If the fetch is successful,
/// the member data is updated in the local cache and then the member data is returned.
///
/// If the fetch also fails, the function returns `None`.
///
/// # Arguments
///
/// * `ctx` - A reference to the context in which this function is called.
/// This context is assumed to contain a local cache and a reference to Skynet for remote fetches.
///
/// * `guild` - A reference to the `GuildId` for which the guild member is desired.
///
/// * `user` - A reference to the `UserId` of the desired guild member.
///
/// # Returns
///
/// * `Option<GuildMember>` - The guild member data if found, `None` otherwise.
///
pub async fn get_member(ctx: &Context, guild: &GuildId, user: &UserId) -> Option<GuildMember> {
    {
        let cache = ctx.cache.read().await;

        if let Some(member) = cache.get_guild_member(guild, user) { return Some(member.clone()) }
    }

    match ctx.skynet.fetch_guild_member(guild, user).await {
        Ok(Ok(member)) => {
            let mut cache = ctx.cache.write().await;
            cache.update_guild_member(&guild, &user, &member);

            Some(member)
        }
        _ => None
    }
}

/// Retrieves a guild from the provided guild ID.
///
/// This function first checks the local cache for the guild.
/// If the guild is found, it is returned immediately.
///
/// If the guild is not found in the local cache, the function tries to fetch the guild
/// from the Skynet (possibly a remote service). If the fetch is successful,
/// the guild data is updated in the local cache and then the guild data is returned.
///
/// If the fetch fails, an error message is logged indicating the failure and the function returns `None`.
///
/// # Arguments
///
/// * `ctx` - A reference to the context in which this function is called.
/// This context is assumed to contain a local cache and a reference to Skynet for remote fetches.
///
/// * `guild` - A reference to the `GuildId` for which the guild is desired.
///
/// # Returns
///
/// * `Option<Guild>` - The guild data if found or successfully fetched, `None` otherwise.
///
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
        Ok(Err(e)) => {
            error!(target: "Runtime", "Failed to fetch guild {:#?}: {:#?}", guild, e);
            None
        }
        Err(e) => {
            error!(target: "Runtime", "Failed to fetch guild {:#?}: {:#?}", guild, e);
            None
        }
    }
}

/// Retrieves a user ID from provided `User` or `GuildMember` options.
///
/// This function checks the user first. If the user is `Some`, it returns
/// the user's ID immediately. If the user is `None`, it checks the guild member.
///
/// If the guild member is `Some` and it contains a user, the function returns
/// the user's ID. If the guild member is either `None` or it does not contain a user,
/// the function returns `None`.
///
/// # Arguments
///
/// * `user` - An `Option<User>` from which to try to get the User ID.
///
/// * `member` - An `Option<GuildMember>` from which to try to get the User ID if the `user` is `None`.
///
/// # Returns
///
/// * `Option<UserId>` - The `UserId` if found, `None` otherwise.
///
pub fn get_user_id(user: &Option<User>, member: &Option<GuildMember>) -> Option<UserId> {
    if let Some(u) = user { return Some(u.id.clone() ) }
    if let Some(m) = member {
        if let Some(u) = &m.user {
            return Some(u.id.clone())
        }
    }
    None
}

/// Retrieves an application.
///
/// This function first checks the local cache for the application.
/// If the application is found, it is returned immediately.
///
/// If the application is not found in the local cache, the function tries to fetch the application
/// from the Skynet (possibly a remote service). If the fetch is successful,
/// the application data is updated in the local cache and the application data is then returned.
///
/// If the fetch fails, the function returns `None`.
///
/// # Arguments
///
/// * `ctx` - A reference to the context in which this function is called.
/// This context is assumed to contain a local cache and a reference to Skynet for remote fetches.
///
/// # Returns
///
/// * `Option<Application>` - The application data if found or successfully fetched, `None` otherwise.
///
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

/// Retrieves the client user.
///
/// This function first checks the local cache for the client user.
/// If the client user is found, it is returned immediately.
///
/// If the client user is not found in the local cache, the function tries to fetch the client user
/// from the Skynet (possibly a remote service). If the fetch is successful,
/// the client user data is updated in the local cache and the client user data is then returned.
/// After updating the cache, the cache's write lock is explicitly dropped,
/// and another read operation is performed on the cache, where the fetched client user data is logged.
///
/// If the fetch fails, the function returns `None`.
///
/// # Arguments
///
/// * `ctx` - A reference to the context in which this function is called.
/// This context is assumed to contain a local cache and a reference to Skynet for remote fetches.
///
/// # Returns
///
/// * `Option<ClientUser>` - The client user data if found or successfully fetched, `None` otherwise.
///
pub async fn get_client_user(ctx: &Context) -> Option<ClientUser> {
    {
        let cache = ctx.cache.read().await;

        if let Some(client_user) = cache.get_client_user() { return Some(client_user.clone()) }
    }

    match ctx.skynet.fetch_client_user().await {
        Ok(Ok(client_user)) => {
            let mut cache = ctx.cache.write().await;
            cache.update_client_user(&client_user);
            drop(cache);

            {
                let cache = ctx.cache.read().await;
                dbg!(&cache.get_client_user());
            }

            Some(client_user)
        }
        _ => None
    }
}


// QUERY STRING SYSTEM

/// Type alias for a `HashMap` where the key and value are both `String`.
/// This type alias is typically used to represent query parameters, where the keys are parameter names
/// and the values are their associated values.
///
pub type QueryParams = HashMap<String, String>;

/// Parses a given query string into a `HashMap` of key-value pairs.
///
/// This function checks if the string contains an '&' symbol. If no '&' is found,
/// an empty `HashMap` is returned. If '&' is found, it splits the string on each '&' symbol to get
/// key-value pairs.
///
/// Each pair is further split on the '=' symbol to get the key and the value. If for some reason the pair
/// does not contain a single '=', an error is returned. Otherwise, the key and value are inserted into
/// the `HashMap`.
///
/// # Arguments
///
/// * `string` - The query string to be parsed into a `HashMap`.
///
/// # Returns
///
/// * `Result<QueryParams>` - If the parsing is successful, `Ok` variant containing the `HashMap` is returned,
/// otherwise an `Error` variant indicating the reason for failure is returned.
pub fn querify(string: String) -> Result<QueryParams> {
    if !string.contains('&') {
        return Ok(HashMap::new())
    }

    let mut map = HashMap::new();
    for pair in string.split('&') {
        let mut it = pair.split('=');

        let kv = match (it.next(), it.next()) {
            (Some(k), Some(v)) => (k.to_string(), v.to_string()),
            (a, b) =>
                return Err(Error::Event(EventError::Runtime(format!("Invalid querystring was received: ({a:?}, {b:?})")))),
        };
        map.insert(kv.0, kv.1);
    }
    Ok(map)
}