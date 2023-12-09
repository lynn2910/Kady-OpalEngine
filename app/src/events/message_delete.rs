use log::{error, warn};
use client::manager::events::Context;
use client::models::components::Color;
use client::models::components::embed::{Author, Embed, Footer};
use client::models::events::MessageDelete;
use client::models::message::{Message, MessageBuilder};
use database::{Database, model};
use translation::message;
use translation::fmt::formatter::Formatter;

pub(crate) async fn triggered(ctx: &Context, payload: MessageDelete) {
    let message = {
        let cache = ctx.cache.read().await;
        let m = cache.get_message(&payload.channel_id, &payload.id).cloned();
        if let Some(m) = m { m } else {
            match ctx.skynet.fetch_message(&payload.channel_id, &payload.id).await {
                Err(e) => {
                    warn!(target: "MessageDeleteEvent", "Error triggered while fetching a deleted message: {e:#?}");
                    return;
                },
                Ok(Err(_)) => return,
                Ok(Ok(m)) => m
            }
        }
    };

    // add the user to the cache
    {
        let mut cache = ctx.cache.write().await;

        cache.update_user(&message.author);
    }

    if payload.guild_id.is_none() { return; }

    let database = ctx.get_data::<Database>().await.expect("No database found");

    // Fetch guild data
    let guild_data = {
        let pool = database.get_pool().await;
        match model::guild::Guild::from_pool(&pool, database.get_requests().await.guilds.get.as_str(), &payload.guild_id.expect("No guild ID found")).await {
            Ok(guild_data) => guild_data,
            Err(e) => {
                error!("Error while fetching guild data: {:?}", e);
                return;
            }
        }
    };

    // The ghostping feature
    let async1 = ghostping(ctx, &message, &guild_data);

    // Wait for all async functions to finish
    async1.await;
}

async fn ghostping(ctx: &Context, message: &Message, guild_data: &model::guild::Guild) {
    // Check if the feature is enabled
    if !guild_data.ghostping_enabled.unwrap_or(false) { return; }

    if let Some(ghostping) = features::ghostping::GhostPing::from_message(message) {
        println!("Ghostping detected with {:?} users mentioned", ghostping.mentions.len());

        // Check if we have one or more mentions
        if ghostping.mentions.is_empty() {
            return;
        }

        let client_user = ctx.get_client_user().await.unwrap();

        let mut embed = Embed::new()
            .set_author(
                Author::new()
                    .set_name(message!("fr", "feature::ghostping::author"))
                    .set_icon_url(client_user.avatar_url(1024, false, "png"))
            )
            .set_footer(Footer::new().set_text(message!("fr", "const::copyright")))
            .set_color(Color::from_hex(message!("fr", "const::palette::main").to_string()));

        // define the embed description
        embed = if ghostping.mentions.len() == 1 {
            // only one user
            let mention = &ghostping.mentions[0];
            if mention.count > 1 {
                embed.set_description(
                    message!(
                        "fr",
                        "feature::ghostping::multiple",
                        Formatter::new()
                            .add("mention", mention.user.to_string())
                            .add("count", mention.count.to_string())
                            .add("author", message.author.id.clone())
                    )
                )
            } else {
                embed.set_description(
                    message!(
                        "fr",
                        "feature::ghostping::single",
                        Formatter::new()
                            .add("mention", mention.user.to_string())
                            .add("author", message.author.id.clone())
                    )
                )
            }
        } else {
            embed.set_description(
                message!(
                    "fr",
                    "feature::ghostping::a_lot",
                    Formatter::new()
                        .add("count", ghostping.mentions.len().to_string())
                        .add("author", message.author.id.clone())
                        .add("mentions", ghostping.mentions.iter().map(|m| format!("<@{}>", m.user)).collect::<Vec<_>>().join(", "))
                )
            )
        };

        // send the embed
        if let Err(e) = ctx.skynet.send_message(&message.channel_id, MessageBuilder::new().add_embed(embed), None).await {
            error!("Failed to send ghostping embed: {:?}", e);
        };
    }
}
