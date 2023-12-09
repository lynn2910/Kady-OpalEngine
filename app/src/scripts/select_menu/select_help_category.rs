use client::manager::events::Context;
use client::models::events::InteractionCreate;
use client::models::message::MessageBuilder;
use translation::message;
use crate::application_commands_manager::CommandType;
use crate::assets::help::{generate_all_commands_message, generate_category_message};
use crate::scripts::{get_guild_locale, QueryParams};

pub async fn triggered(ctx: &Context, payload: &InteractionCreate, _: QueryParams) {
    // we can unwrap safely because of a check in the event listener
    let interaction_data = payload.interaction.data.as_ref()
        .unwrap();

    let value = interaction_data.values.as_ref()
        .map(|v| v.get(0))
        .unwrap_or(None);

    let v = value.map(CommandType::try_from)
        .unwrap_or(Err(()));

    match v {
        Err(_) | Ok(CommandType::Dev) => {
            let _ = payload.interaction.reply(
                &ctx.skynet,
                MessageBuilder::new()
                    .set_content(
                        message!(
                            get_guild_locale(&payload.interaction.guild_locale),
                            "slashs::help::invalid_category"
                        )
                    )
            ).await;
        }
        Ok(ctg) => {
            match ctg {
                CommandType::All => {
                    let _ = payload.interaction.edit_original_message(
                        &ctx.skynet,
                        generate_all_commands_message(
                            ctx,
                            &get_guild_locale(&payload.interaction.guild_locale)
                        ).await
                    ).await;
                },
                _ => {
                    let _ = payload.interaction.edit_original_message(
                        &ctx.skynet,
                        generate_category_message(
                            ctx,
                            ctg,
                            &get_guild_locale(&payload.interaction.guild_locale)
                        ).await
                    ).await;
                }
            }
        }
    }
}