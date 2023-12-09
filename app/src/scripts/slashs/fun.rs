pub(crate) mod rateit {
    use rand::Rng;
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::scripts::get_guild_locale;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);
        let note = {
            let mut thread_rng = rand::thread_rng();

            thread_rng.gen_range(0..20)
        };

        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content(
                    message!(
                        local,
                        "slashs::rateit::msg",
                        Formatter::new()
                            .add("n", note)
                    )
                )
        ).await;
    }
}

pub(crate) mod unacceptable {
    use client::manager::events::Context;
    use client::models::components::message_components::{ActionRow, Button, ButtonStyle, Component};
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let _ = payload.interaction.reply(
            &ctx.skynet,
            MessageBuilder::new()
                .set_content("https://cdn.discordapp.com/attachments/426802397667655680/825126979418324992/UNACCEPTABLE_1.mp4")
                .add_component(
                    Component::ActionRow(
                        ActionRow::new()
                            .add_component(
                                Component::Button(
                                    Button::new("")
                                        .set_style(ButtonStyle::Link)
                                        .set_label("Hello ?")
                                        .set_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                                        .set_disabled(false)
                                )
                            )
                    )
                )
        ).await;
    }
}

pub(crate) mod eight_ball {
    use rand::Rng;
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::scripts::{get_guild_locale, get_user, get_user_id};
    use crate::crates::error_broadcaster::*;
    use crate::broadcast_error;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);

        let choice = {
            let mut thread_rng = rand::thread_rng();
            let n = thread_rng.gen_range(0..=7);

            translation::fmt::translate(&local, format!("slashs::8ball::choices::{n}").as_str(), &Formatter::new()).to_string()
        };

        let author = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => match get_user(ctx, &id).await {
                Some(u) => u,
                None => {
                    let _ = payload.interaction.reply(
                        &ctx.skynet,
                        MessageBuilder::new().set_content(message!(local, "errors::cannot_acquire_user"))
                    ).await;
                    return;
                }
            }
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new().set_content(message!(local, "errors::cannot_get_user_id"))
                ).await;

                broadcast_error!(
                    localisation: BroadcastLocalisation::default()
                        .set_guild(payload.interaction.guild_id.clone())
                        .set_channel(payload.interaction.channel_id.clone())
                        .set_code_path("app/src/scripts/slashs/fun.rs:8ball:104"),
                    interaction: BroadcastInteraction::default()
                        .set_name("8ball")
                        .set_type(BroadcastInteractionType::SlashCommand),
                    details: BroadcastDetails::default()
                        .add("reason", "Cannot acquire the user ID"),
                    ctx.skynet.as_ref()
                );

                return;
            }
        };

        let question = payload.interaction.data.as_ref()
            .map(|d| {
                d.options.as_ref().map(|options| {
                    options.iter().find(|o| o.name.as_str() == "question")
                        .map(|q| q.value.as_ref().map(|v| v.to_string()))
                        .unwrap_or(None)
                }).unwrap_or(None)
            })
            .unwrap_or(None);

        if let Some(q) = question {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            &local,
                            "slashs::8ball::msg",
                            Formatter::new()
                                .add("author", author.global_name.unwrap_or(author.username))
                                .add("question", q)
                                .add("answer", choice)
                        )
                    )
            ).await;
        } else {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            &local,
                            "slashs::8ball::without_question",
                            Formatter::new()
                                .add("author", author.global_name.unwrap_or(author.username))
                                .add("answer", choice)
                        )
                    )
            ).await;
        }
    }
}