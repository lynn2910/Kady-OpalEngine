use client::manager::events::Context;
use client::models::components::message_components::{ActionRow, Component, ComponentType, TextInput, TextInputStyle};
use client::models::events::InteractionCreate;
use client::models::message::MessageBuilder;
use translation::message;
use crate::scripts::get_guild_locale;

pub(in crate::scripts) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
    let local = get_guild_locale(&payload.interaction.guild_locale);

    let _ = payload.interaction.reply_with_modal(
        &ctx.skynet,
        MessageBuilder::new()
            .set_title(message!(&local, "features::cookies::quiz::modal::placeholder").to_string())
            .set_custom_id("COOKIE_USER_QUIZ_ANSWER")
            .add_component(
                Component::ActionRow(
                    ActionRow::new()
                        .add_component(Component::TextInput(
                            TextInput {
                                kind: ComponentType::TextInput,
                                style: Some(TextInputStyle::Paragraph),
                                label: Some(
                                    message!(&local, "features::cookies::quiz::modal::question").to_string()
                                ),
                                placeholder: Some(
                                    message!(&local, "features::cookies::quiz::modal::placeholder").to_string()
                                ),
                                custom_id: "COOKIE_USER_QUIZ_ANSWER_FIELD".into(),
                                min_length: Some(1),
                                max_length: Some(512),
                                disabled: None,
                                value: None,
                                required: true
                            }
                        ))
                )
            )
    ).await;
}