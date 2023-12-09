pub(crate) mod suggestion {
    use client::manager::events::Context;
    use client::models::events::InteractionCreate;

    pub(in crate::scripts) async fn triggered(ctx: &Context, payload: &InteractionCreate) {
        let _ = payload.interaction.defer(&ctx.skynet, None).await;

        let interaction_data = payload.interaction.data.as_ref().unwrap();

        dbg!(&interaction_data);
    }
}