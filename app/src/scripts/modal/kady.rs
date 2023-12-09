pub(crate) mod suggest {
    use log::error;
    use client::manager::events::Context;
    use client::models::channel::ChannelId;
    use client::models::components::Color;
    use client::models::components::embed::{Author, Embed, Field};
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use config::Config;
    use translation::message;
    use crate::constants::DEFAULT_AVATAR;
    use crate::scripts::{get_guild_locale, get_user, get_user_id};
    use crate::scripts::modal::get_modal_textinput;
    use crate::scripts::slashs::internal_error;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);
        let interaction_data = payload.interaction.data.as_ref().unwrap();

        let fields = get_modal_textinput(&interaction_data.components.as_ref().unwrap_or(&Vec::new()));

        let config = match ctx.get_data::<Config>().await {
            Some(d) => d,
            None => return internal_error(ctx, &payload.interaction, local, "31001").await
        };

        let suggest_channel_id: ChannelId = config.client.suggestion_channel.into();

        let user = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => get_user(&ctx, &id).await,
            None => None
        };

        let m = suggest_channel_id.send_message(
            &ctx.skynet,
            MessageBuilder::new()
                .add_embed(
                    Embed::new()
                        .set_color(Color::from_hex("F9F871"))
                        .set_title("Nouvelle suggestion")
                        .add_field(
                            Field::new()
                                .set_name("💡 Suggestion")
                                .set_value(
                                    fields.get("SUGGESTION")
                                        .cloned()
                                        .unwrap_or("CANNOT_GET_CUSTOM_ID--SUGGESTION".into())
                                        .to_string()
                                )
                        )
                        .add_field(
                            Field::new()
                                .set_name("✌️ Être re-contacté:")
                                .set_value(
                                    fields.get("SUGGESTION_CONTACT_AFTER")
                                        .cloned()
                                        .unwrap_or("N'a pas répondu".into())
                                        .to_string()
                                )
                        )
                        .set_author(
                            Author::new()
                                .set_icon_url(
                                    user.as_ref().map(|u| u.avatar_url(512, true, "png"))
                                        .unwrap_or(Some(DEFAULT_AVATAR.to_string()))
                                )
                                .set_name(
                                    user.as_ref().map(|u| u.global_name
                                        .clone()
                                        .unwrap_or(
                                            user.as_ref()
                                                .map(|u| u.username.clone())
                                                .unwrap_or("CANNOT_RESOLVE_USERNAME".into())
                                        )
                                    ).unwrap_or("CANNOT_RESOLVE_USERNAME".into())
                                )
                        )
                )
        ).await;

        if let Err(e) = m {
            error!(target: "SuggestionModal", "Cannot send the suggestion message from the modal interaction: {e:#?}");
            internal_error(&ctx, &payload.interaction, local, "31002").await;
        } else {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(local, "slashs::kady::suggest::sent"))
                    .set_ephemeral(true)
            ).await;
        }
    }
}

pub(crate) mod issue {
    use log::error;
    use client::manager::events::Context;
    use client::models::channel::ChannelId;
    use client::models::components::{Color, Emoji};
    use client::models::components::embed::{Author, Embed, Field};
    use client::models::components::message_components::{ActionRow, Button, ButtonStyle, Component};
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use config::Config;
    use translation::message;
    use crate::constants::DEFAULT_AVATAR;
    use crate::scripts::{get_guild_locale, get_user, get_user_id};
    use crate::scripts::modal::get_modal_textinput;
    use crate::scripts::slashs::internal_error;

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);
        let interaction_data = payload.interaction.data.as_ref().unwrap();

        let fields = get_modal_textinput(&interaction_data.components.as_ref().unwrap_or(&Vec::new()));

        let config = match ctx.get_data::<Config>().await {
            Some(d) => d,
            None => return internal_error(ctx, &payload.interaction, local, "32001").await
        };

        let issue_report_channel_id: ChannelId = config.client.issue_channel.into();

        let user = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => get_user(&ctx, &id).await,
            None => None
        };

        let m = issue_report_channel_id.send_message(
            &ctx.skynet,
            MessageBuilder::new()
                .add_embed(
                    Embed::new()
                        .set_color(Color::from_hex("FF6F91"))
                        .set_title("Nouveau signalement de problème")
                        .add_field(
                            Field::new()
                                .set_name("🗒️ Type du bug")
                                .set_value(
                                    fields.get("BUG_TYPE")
                                        .cloned()
                                        .unwrap_or("CANNOT_GET_CUSTOM_ID--BUG_TYPE".into())
                                        .to_string()
                                )
                        )
                        .add_field(
                            Field::new()
                                .set_name("🐛 Description du bug")
                                .set_value(
                                    fields.get("BUG_DESCRIPTION")
                                        .cloned()
                                        .unwrap_or("CANNOT_GET_CUSTOM_ID--BUG_DESCRIPTION".into())
                                        .to_string()
                                )
                        )
                        .add_field(
                            Field::new()
                                .set_name("✌️ Être re-contacté:")
                                .set_value(
                                    fields.get("SUGGESTION_CONTACT_AFTER")
                                        .cloned()
                                        .unwrap_or("N'a pas répondu".into())
                                        .to_string()
                                )
                        )
                        .set_author(
                            Author::new()
                                .set_icon_url(
                                    user.as_ref().map(|u| u.avatar_url(512, true, "png"))
                                        .unwrap_or(Some(DEFAULT_AVATAR.to_string()))
                                )
                                .set_name(
                                    user.as_ref().map(|u| u.global_name
                                        .clone()
                                        .unwrap_or(
                                            user.as_ref()
                                                .map(|u| u.username.clone())
                                                .unwrap_or("CANNOT_RESOLVE_USERNAME".into())
                                        )
                                    ).unwrap_or("CANNOT_RESOLVE_USERNAME".into())
                                )
                        )
                )
        ).await;

        if let Err(e) = m {
            error!(target: "SuggestionModal", "Cannot send the issue message from the modal interaction: {e:#?}");
            internal_error(&ctx, &payload.interaction, local, "32002").await;
        } else {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(&local, "slashs::kady::issue::sent"))
                    .set_ephemeral(true)
                    .add_component(
                        Component::ActionRow(
                            ActionRow::new()
                                .add_component(
                                    Component::Button(
                                        Button::new("")
                                            .set_url(config.client.support_url)
                                            .set_emoji(Emoji::new(None, "🔗"))
                                            .set_label(message!(local, "slashs::kady::issue::sent_btn"))
                                            .set_style(ButtonStyle::Link)
                                    )
                                )
                        )
                    )
            ).await;
        }
    }
}

pub(crate) mod review {
    use std::str::FromStr;
    use chrono::Utc;
    use log::error;
    use regex::Regex;
    use client::manager::events::Context;
    use client::models::channel::ChannelId;
    use client::models::components::Color;
    use client::models::components::embed::{Author, Embed, Field};
    use client::models::events::InteractionCreate;
    use client::models::message::MessageBuilder;
    use config::Config;
    use translation::fmt::formatter::Formatter;
    use translation::message;
    use crate::constants::DEFAULT_AVATAR;
    use crate::scripts::{get_guild_locale, get_user, get_user_id};
    use crate::scripts::modal::get_modal_textinput;
    use crate::scripts::slashs::internal_error;

    fn parse_note(source: String) -> Option<String> {
        let reg = Regex::from_str(r#"^([0-5])\/5|([0-5])|(⭐|\*){1,5}$"#).unwrap();

        let captures = reg.captures(source.as_str());
        if let Some(captures) = captures {
            Some(
                captures.get(0)
                    .map(|c| c.as_str().to_string())
                    .unwrap_or(source)
            )
        } else if source.contains('-') {
            None
        } else {
            Some(source)
        }
    }

    pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let local = get_guild_locale(&payload.interaction.guild_locale);
        let interaction_data = payload.interaction.data.as_ref().unwrap();

        let fields = get_modal_textinput(&interaction_data.components.as_ref().unwrap_or(&Vec::new()));

        let config = match ctx.get_data::<Config>().await {
            Some(d) => d,
            None => return internal_error(ctx, &payload.interaction, local, "33001").await
        };

        let issue_report_channel_id: ChannelId = config.client.issue_channel.into();

        let user = match get_user_id(&payload.interaction.user, &payload.interaction.member) {
            Some(id) => get_user(&ctx, &id).await,
            None => None
        };

        let review = fields.get("REVIEW")
            .cloned()
            .unwrap_or("CANNOT_GET_CUSTOM_ID--REVIEW".into())
            .to_string();

        let raw_note = fields.get("NOTE")
            .cloned()
            .unwrap_or("CANNOT_GET_CUSTOM_ID--NOTE".into())
            .to_string();

        let note = match parse_note(raw_note) {
            Some(n) => n,
            None => {
                let _ = payload.interaction.reply(
                    &ctx.skynet,
                    MessageBuilder::new()
                        .set_content(
                            message!(
                                &local,
                                "slashs::kady::review::negative",
                                Formatter::new().add("support", config.client.support_url)
                            )
                        )
                        .set_ephemeral(true)
                ).await;
                return;
            }
        };

        let m = issue_report_channel_id.send_message(
            &ctx.skynet,
            MessageBuilder::new()
                .add_embed(
                    Embed::new()
                        .set_color(Color::from_hex("FFAC33"))
                        .set_description(format!("> ✨ {review:.1990}"))
                        .set_timestamp(Utc::now())
                        .add_field(
                            Field::new()
                                .set_name("⭐ Note")
                                .set_value(format!("> {note}"))
                        )
                        .set_author(
                            Author::new()
                                .set_icon_url(
                                    user.as_ref().map(|u| u.avatar_url(512, true, "png"))
                                        .unwrap_or(Some(DEFAULT_AVATAR.to_string()))
                                )
                                .set_name(
                                    user.as_ref().map(|u| u.global_name
                                        .clone()
                                        .unwrap_or(
                                            user.as_ref()
                                                .map(|u| u.username.clone())
                                                .unwrap_or("CANNOT_RESOLVE_USERNAME".into())
                                        )
                                    ).unwrap_or("CANNOT_RESOLVE_USERNAME".into())
                                )
                        )
                )
        ).await;

        if let Err(e) = m {
            error!(target: "SuggestionModal", "Cannot send the review message from the modal interaction: {e:#?}");
            internal_error(&ctx, &payload.interaction, local, "33002").await;
        } else {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(message!(&local, "slashs::kady::review::sent"))
                    .set_ephemeral(true)
            ).await;
        }
    }
}