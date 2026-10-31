use crate::{CommandResult, Context};

#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> CommandResult {
    ctx.reply("Pong!").await?;

    Ok(())
}
