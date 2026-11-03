use crate::{utilities::guild, voice};
use poise::Context::Application;

use crate::{CommandResult, Context};

use super::get_theme_path;

#[poise::command(slash_command, prefix_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "intro or outro"] kind: String,
    #[description = "Specific intro or outro to play, if you know the file name and have more than one"]
    name: Option<String>,
) -> CommandResult {
    let _ = ctx.defer_ephemeral().await;

    let data = ctx.data();
    let member = ctx
        .author_member()
        .await
        .expect("User is a member of the server this command is run in");

    let voice_channel = match guild::get_voice_channel(ctx, &member).await {
        Ok(voice_channel) => voice_channel,
        Err(err) => {
            let _ = ctx.say("Get in a voice channel idot.").await;
            return Err(err.into());
        }
    };

    let tag = member.user.tag();
    let storage_client = &data.storage_client;

    let path = get_theme_path(&tag, &kind, name.as_deref(), storage_client).await?;
    let file_stream = storage_client.get_stream(&path).await?;

    let _ = voice::play(ctx, voice_channel.id, voice_channel.guild_id, file_stream).await;

    // If application command, then remove the initial interaction response
    if let Application(app_ctx) = ctx {
        let _ = app_ctx.interaction.delete_response(ctx).await;
    }

    Ok(())
}
