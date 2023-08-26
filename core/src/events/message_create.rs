use log::error;
use client::manager::events::Context;
use client::models::events::MessageCreate;
use client::models::message::MessageBuilder;
use database::Database;
use database::model::guild::Guild;
use database::model::users::User;
use translation::fmt::formatter::Formatter;
use translation::message;

pub(crate) async fn triggered(ctx: Context, payload: MessageCreate) {
    // add the user to the cache
    {
        let mut cache = ctx.cache.write().await;

        cache.update_user(&payload.message.author);
    }


    let db = ctx.get_data::<Database>().await.expect("Cannot acquire the Database structure, wtf?");
    let pool = db.get_pool().await;

    let requests = db.get_requests().await;


    // register the use in the database
    {
        match User::ensure(&pool, requests.users.ensure.as_str(), payload.message.author.id.to_string()).await {
            Ok(()) => (),
            Err(e) => {
                error!(target: "Runtime", "An error occured while ensuring the presence of the author in the database from the message_create event: {e:#?}")
            }
        }
    }

    if let Some(guild_id) = &payload.guild_id {
        match Guild::from_pool(&pool, requests.guilds.get.as_str(), guild_id).await {
            Ok(d) => {
                let xp_result = features::xp::trigger(
                    &ctx,
                    &d,
                    &pool,
                    &requests,
                    &payload.message.author,
                    &payload.message.channel_id
                ).await;

                match xp_result {
                    Ok(_) => {}
                    Err(code) => {
                        let _ = payload.message.channel_id.send_message(
                            &ctx.skynet,
                            MessageBuilder::new()
                                .set_content(
                                    message!(
                                        d.lang,
                                        "errors::internal_error",
                                        Formatter::new().add("code", format!("01{code:03}"))
                                    )
                                )
                        ).await;
                    }
                }
            },
            Err(e) => {
                error!(target: "Runtime", "cannot acquire the guild informations in the MessageCreate event: {e:#?}");
            }
        };
    }
}