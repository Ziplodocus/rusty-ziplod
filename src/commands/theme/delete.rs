use crate::{CommandResult, Context};

#[poise::command(slash_command, prefix_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "intro or outro"] kind: String,
    #[description = "intro/outro to delete"] name: String,
) -> CommandResult {
    let _ = ctx.defer_ephemeral().await;
    let storage_client = &ctx.data().storage_client;
    let user = ctx.author();

    let tag = user.tag();

    let path = format!("themes/{tag}/{kind}/{name}.mp3");

    let _ = match storage_client.delete(path.as_str()).await {
        Ok(_) => ctx.say(format!("Deleted {kind} {name}")).await,
        Err(err) => ctx.say(err.to_string()).await,
    };

    Ok(())
}
