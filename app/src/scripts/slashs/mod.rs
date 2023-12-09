use client::manager::events::Context;
use client::models::interaction::Interaction;
use client::models::message::MessageBuilder;
use translation::{message, fmt::formatter::Formatter};

pub(crate) mod citation;
pub(crate) mod admin;
pub(crate) mod xp;
pub(crate) mod top;
pub(crate) mod cookies;
pub(crate) mod common;
pub(crate) mod fun;

#[allow(unused)]
pub async fn internal_error(ctx: &Context, interaction: &Interaction, local: impl ToString, code: impl ToString) {
    let _ = interaction.reply(
        &ctx.skynet,
        MessageBuilder::new()
            .set_content(
                message!(
                    local.to_string(),
                    "errors::internal_error",
                    Formatter::new().add("code", code.to_string())
                )
            )
            .set_ephemeral(true)
    ).await;
}

pub async fn internal_error_deferred(ctx: &Context, interaction: &Interaction, local: impl ToString, code: impl ToString) {
    let _ = interaction.update(
        &ctx.skynet,
        MessageBuilder::new()
            .set_content(
                message!(
                    local.to_string(),
                    "errors::internal_error",
                    Formatter::new().add("code", code.to_string())
                )
            )
            .set_ephemeral(true)
    ).await;
}