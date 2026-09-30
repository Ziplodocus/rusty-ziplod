use serenity::{
    framework::standard::{CommandResult, macros::command},
    model::prelude::Message,
    prelude::Context,
};

#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}
