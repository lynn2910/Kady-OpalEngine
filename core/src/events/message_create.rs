use log::error;
use regex::Regex;
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

    // blep
    if let Some(c) = payload.message.content.as_ref().map(|c| c.trim().to_lowercase()) {
        let c = {
            let reg: Regex = Regex::new(r"\s+").unwrap();
            reg.replace(c.as_str(), " ").to_string()
        };

        let reply = match c.as_str() {
            "riki like fighting easy monsters" => Some("Dundun"),
            "i'm really feeling it" | "i'm really feeling it!" => Some("Said shulk!"),
            "hear that, noah? lanz wants something a bit meatier"
                | "hear that, noah? lanz wants something a bit meatier!"=> Some("Eunie is as Eunie does"),
            "sometimes, you just gotta get wild" => Some("*pulls out the monado*"),
            "i am Dunban, attack me if you dare" | "i am Dunban, attack me if you dare!" => Some("Sh*ts is about to go wild!"),
            "maybe we'll survive after all" => Some("I like your attitude!"),
            "salvager code" => Some(r#"
## The Salvager's Code
*by rex*

1) Swim life a fish, drint like one too!
2) Always help others that help you!
3) Make a girl cry ? That's not gonna fly; Make a girl smile ? You pass the trial!
4) Open a chest, it might turn out great. UNtil then it's just a crate
5) Always be closing.
6) First have a punch-out, then drink to forget. Once you've forgotten, the freidnship's all set
7) Never leave a debt unpaid
"#),
            "double spinning edge" | "double spinning edge!" => Some("Double spinning edge!!\n*Steal the agro and die*"),
            _ => None
        };

        if let Some(r) = reply {
            let _ = payload.message.channel_id.send_message(&ctx.skynet, MessageBuilder::new().set_content(r)).await;
        }
    }
}