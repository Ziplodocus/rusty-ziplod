use crate::{CommandResult, Context};
use serenity::model::channel::Attachment;

#[poise::command(slash_command, prefix_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "intro or outro"] kind: String,
    #[description = "intro/outro name (for your reference)"] name: String,
    #[description = "mp3 file to upload as your theme"] file: Attachment,
) -> CommandResult {
    let _ = ctx.defer_ephemeral().await;
    let data = ctx.data();
    let user = ctx.author();

    let content_type = match file.content_type.as_deref() {
        Some("audio/mpeg") => kind.to_owned(),
        Some("audio/mpeg3") => kind.to_owned(),
        _ => {
            let _ = ctx
                .say("MP3 FILE. Everything else is garbage. Like your mother.")
                .await;
            return Ok(());
        }
    };

    if file.size > 3347520 {
        let _ = ctx
            .say("No one wants to hear your life story. Keep it short and sweet.")
            .await;
        return Ok(());
    }

    let stream = reqwest::Client::new()
        .get(&file.url)
        .send()
        .await
        .expect("oh well")
        .bytes_stream();

    let storage_client = &data.storage_client;

    let path = format!("themes/{}/{}/{}.mp3", user.tag(), kind, name);

    let res = storage_client
        .create_stream(stream, &path, None, &content_type)
        .await;

    let _ = match res {
        Ok(item) => ctx.reply(format!("Added {} {}", kind, name)).await,
        Err(err) => {
            dbg!(err);
            ctx.reply(format!(
                "Failed to add {} {}, what have you done dingus",
                kind, name
            ))
            .await
        }
    };

    Ok(())
}
