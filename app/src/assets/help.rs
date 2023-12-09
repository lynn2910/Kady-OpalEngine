use client::manager::events::Context;
use client::models::components::Color;
use client::models::components::embed::{Author, Embed, Field, Footer};
use client::models::components::message_components::{ActionRow, Component, SelectMenu, SelectOption};
use client::models::interaction::ApplicationCommand;
use client::models::message::MessageBuilder;
use config::Config;
use translation::fmt::formatter::Formatter;
use translation::message;
use crate::application_commands_manager;
use crate::application_commands_manager::{COMMANDS, CommandType};
use crate::constants::DEFAULT_AVATAR;
use crate::scripts::get_client_user;

pub(crate) async fn generate_default_message(
    ctx: &Context,
    local: &String
) -> Result<MessageBuilder, (usize, String)>
{
    let config = match ctx.get_data::<Config>().await {
        Some(c) => c,
        None => return Err((0, "Cannot obtain the config from the context".into()))
    };
    let cache = ctx.cache.read().await;
    let commands_number = {
        let registered_commands = application_commands_manager::COMMANDS.read().await;

        registered_commands.values()
            .filter(|ctg| ctg.visible)
            .map(|ctg| ctg.commands.len())
            .sum::<usize>()
    };

    let most_used_commands = {
        let all_commands = cache.get_application_commands();

        config.client.most_used_commands.iter()
            .map(|name|
                all_commands.iter().find(|c| &c.name == name)
                    .map(|c| format!("</{n}:{id}>", n = c.name, id = c.id))
                    .unwrap_or(format!("`{name}`"))
            )
            .collect::<Vec<String>>()
    };

    drop(cache);

    let client_user = match get_client_user(ctx).await {
        Some(a) => a,
        None => return Err((1, "Cannot obtain the client's user :(".into()))
    };

    Ok(
        MessageBuilder::new()
            .add_embed(
                Embed::new()
                    .set_color(
                        Color::from_hex(message!(local, "const::palette::main"))
                    )
                    .set_author(
                        Author::new()
                            .set_icon_url(
                                Some(
                                    client_user
                                        .avatar_url(512, false, "png")
                                        .unwrap_or(DEFAULT_AVATAR.to_string())
                                )
                            )
                            .set_name(
                                message!(local, "slashs::help::author_text")
                            )
                    )
                    .set_description(
                        message!(
                            local,
                            "slashs::help::general_message",
                            Formatter::new()
                                .add("most_used_commands", most_used_commands.join(", "))
                                .add("commands_number", commands_number)
                        )
                    )
            )
            .add_component(
                Component::ActionRow(
                    ActionRow::new()
                        .add_component(
                            generate_category_selector(local).await
                        )
                )
            )
    )
}

pub(crate) async fn generate_category_selector(local: &String) -> Component {
    let categories = application_commands_manager::COMMANDS.read().await;

    let mut select_menu = SelectMenu::new("SELECT_HELP_CATEGORY")
        .set_disabled(false)
        .set_max_values(1)
        .set_min_values(1)
        .set_placeholder(
            Some(message!(local, "slashs::help::category_selector::placeholder"))
        )
        .add_option(
            SelectOption::new(
                message!(local, "slashs::help::category_selector::categories::all::name"),
                "all"
            ).set_description(
                Some(message!(local, "slashs::help::category_selector::categories::all::description")),
            ).set_default(false)
                .set_emoji(Some("🌐"))
        );

    for (ctg_type, category) in categories.iter() {
        if !category.visible { continue; }

        let ctg_id: &str = ctg_type.into();

        let description = {
            let d  = translation::fmt::translate(local, format!("slashs::help::category_selector::categories::{ctg_id}::description").as_str(), &Formatter::new());

            if d.is_value_none() {
                None
            } else {
                Some(d.to_string())
            }
        };

        select_menu = select_menu.add_option(
            SelectOption::new(
                translation::fmt::translate(local, format!("slashs::help::category_selector::categories::{ctg_id}::name").as_str(), &Formatter::new()),
                ctg_id
            )
                .set_description(description)
                .set_default(false)
                .set_emoji(Some(category.emoji.clone()))
        );
    }

    Component::SelectMenu(select_menu)
}


pub(crate) async fn generate_all_commands_message(
    ctx: &Context,
    local: &String
) -> MessageBuilder
{
    let client_user = get_client_user(ctx).await;

    let cache = ctx.cache.read().await;
    let application_commands = cache.get_application_commands();

    let mut embed = Embed::new()
        .set_author(
            Author::new()
                .set_name(
                    format!(
                        "{} Help",
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
        .set_title(
            message!(
                local,
                "slashs::help::all_commands::title"
            )
        )
        .set_description(
            message!(
                local,
                "slashs::help::all_commands::description",
                Formatter::new()
                    .add("commands_number", application_commands.len())
            )
        );


    let commands = COMMANDS.read().await;

    // TODO add command name localisation
    for (id, category) in commands.iter() {
        if !category.visible { continue; }

        let ctg_id: &str = id.into();

        let category_name = translation::fmt::translate(
            local,
            format!("slashs::help::category_selector::categories::{ctg_id}::name").as_str(),
            &Formatter::new()
        ).to_string();

        embed = embed.add_field(
            Field::new()
                .set_name(format!("> **{category_name}**"))
                .set_value(
                    category.commands
                        .iter()
                        .map(|(name, c)|
                            find_command_from_name(&application_commands, name.as_str())
                                .map(|c| format!("</{n}:{id}>", n = c.name, id = c.id))
                                .unwrap_or(format!("`{}`", c.name))
                        )
                        .collect::<Vec<String>>()
                        .join(", ")
                )
        )
    }

    drop(application_commands);

    MessageBuilder::new()
        .add_embed(embed)
        .add_component(
            Component::ActionRow(
                ActionRow::new()
                    .add_component(
                        generate_category_selector(local).await
                    )
            )
        )
}

fn find_command_from_name<'a>(
    slash_commands: &'a [&ApplicationCommand],
    search: &str
) -> Option<&'a&'a ApplicationCommand>
{
    slash_commands.iter()
        .find(|c| c.name.eq(search))
}

pub(crate) async fn generate_category_message(
    ctx: &Context,
    category: CommandType,
    local: &String
) -> MessageBuilder
{
    let docker = COMMANDS.read().await;
    let category_commands = match docker.get(&category) {
        Some(c) => c,
        None => {
            return MessageBuilder::new()
                .set_content(
                    message!(local, "slashs::help::invalid_category")
                )
        }
    };

    let cache = ctx.cache.read().await;
    let application_commands = cache.get_application_commands();

    let commands = category_commands.commands
        .iter()
        .map(|(name, c)|
            find_command_from_name(&application_commands, name.as_str())
                .map(|c| format!("</{n}:{id}>", n = c.name, id = c.id))
                .unwrap_or(format!("`{}`", c.name))
        )
        .collect::<Vec<String>>()
        .join(", ");

    drop(application_commands);
    drop(cache);

    let client_user = get_client_user(ctx).await;

    let category_name = translation::fmt::translate(
        local,
        format!("slashs::help::category_selector::categories::{}::name", category.to_string()).as_str(),
        &Formatter::new()
    ).to_string();

    MessageBuilder::new()
        .add_embed(
            Embed::new()
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
                .set_author(
                    Author::new()
                        .set_name(
                            message!(
                                local,
                                "slashs::help::category::author",
                                Formatter::new().add("ctg", category_name.as_str())
                            )
                        )
                        .set_icon_url(
                            client_user.as_ref().map(|c| c.avatar_url(512, false, "png"))
                                .unwrap_or(Some(DEFAULT_AVATAR.to_string()))
                        )
                )
                .set_description(
                    message!(
                        local,
                        "slashs::help::category::description",
                        Formatter::new()
                            .add("ctg", category_name.as_str())
                            .add("cmds", commands)
                    )
                )
        )
}

pub(crate) async fn generate_command_help(
    ctx: &Context,
    category: CommandType,
    command: &str,
    local: &String
) -> MessageBuilder
{
    let category_name = translation::fmt::translate(
        local,
        format!("slashs::help::category_selector::categories::{}::name", category.to_string()).as_str(),
        &Formatter::new()
    ).to_string();

    let client_user = get_client_user(&ctx).await;

    let command_declaration = translation::fmt::translate(
        local,
        format!("commands::{command}").as_str(),
        &Formatter::new()
    );

    let desc = match command_declaration.get_children("description") {
        Some(desc) => format!(
            "\n\n{}",
            message!(
                local,
                "slashs::help::command::if::description",
                Formatter::new().add("desc", desc.to_string())
            ).to_string()
        ),
        None => String::new()
    };
    let use_case = match command_declaration.get_children("use") {
        Some(usc) => format!(
            "\n\n{}",
            message!(
                local,
                "slashs::help::command::if::use",
                Formatter::new().add("use", usc.to_string().replace(';', "\n"))
            ).to_string()
        ),
        None => String::new()
    };

    let required_description = message!(
        local,
        "slashs::help::command::required_description",
        Formatter::new()
            .add("name", command)
            .add("ctg", category_name)
    ).to_string();

    MessageBuilder::new()
        .add_embed(
            Embed::new()
                .set_footer(
                    Footer::new()
                        .set_text(
                            message!(
                                local,
                                "const::copyright"
                            )
                        )
                )
                .set_title(
                    message!(
                        local,
                        "slashs::help::command::title",
                        Formatter::new().add("cmd", command)
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
                .set_author(
                    Author::new()
                        .set_name(
                            format!(
                                "{} Help",
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
                .set_description(
                    format!("{required_description}{desc}{use_case}")
                )
        )
}