use std::cmp::max;
use log::warn;
use client::manager::events::Context;
use client::models::components::message_components::{ActionRow, Button, ButtonStyle, Component};
use client::models::components::Emoji;
use client::models::events::InteractionCreate;
use client::models::message::MessageBuilder;
use translation::fmt::formatter::Formatter;
use translation::message;
use crate::scripts::get_guild_locale;

pub(crate) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
    let local = get_guild_locale(&payload.interaction.guild_locale);

    let shard_manager = ctx.shard_manager.read().await;

    let mut all_latency = Vec::new();
    for (_, shard) in shard_manager.get_shards().iter() {
        let latency = *shard.ping.read().await;
        all_latency.push(latency);
    }
    // sum all latencies and divide by the number of shards
    let median = all_latency.iter().sum::<u128>() / all_latency.len() as u128;

    let actual_shard = shard_manager.get_shard(&payload.shard);
    let actual_shard_latency = if let Some(s) = actual_shard { *s.ping.read().await } else { 0 };

    let msg = {
        if actual_shard_latency == 0 {
            MessageBuilder::new().set_content(message!(local, "slashs::ping::booting"))
        } else {
            MessageBuilder::new()
                .set_content(
                    message!(
                        local.clone(),
                        "slashs::ping::content",
                        Formatter::new()
                            .add("ping", actual_shard_latency.to_string())
                            .add("shard_id", (max(ctx.shard_id, 1)).to_string())
                    )
                )
                .add_component(
                    Component::ActionRow(
                        ActionRow::new().add_component(
                            Component::Button(
                                Button::new("A")
                                    .set_label(message!(
                                        local.clone(),
                                        "slashs::ping::button::label",
                                        Formatter::new().add("median", median.to_string())
                                    ))
                                    .set_emoji(Emoji::new(None, message!(local, "slashs::ping::button::emoji")))
                                    .set_style(ButtonStyle::Secondary)
                                    .set_disabled(true)
                            )
                        )
                    )
                )
        }
    };

    if let Err(e) = payload.interaction.reply(&ctx.skynet, msg).await {
        warn!("Failed to reply to slash command: {:?}", e);
    }
}