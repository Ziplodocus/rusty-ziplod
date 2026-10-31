use crate::{
    Context, errors::Error, storage::StorageClient, utilities::message::fetch_voice_channel, voice,
};
use serenity::all::Message;

use super::{get_tag, get_theme_path};

pub async fn play(ctx: Context<'_>, msg: &Message, mut args: Args) -> Result<(), Error> {
    let data = ctx.data.read().await;

    let kind: Box<str> = match args.single::<String>() {
        Ok(kind) if kind == "intro" || kind == "outro" => kind.into(),
        _ => {
            let _ = msg.reply(ctx, "Try specifying intro or outro dimwit").await;
            return Ok(());
        }
    };

    let voice_channel = match fetch_voice_channel(ctx, ctx.author_member().into()).await {
        Ok(voice_channel) => voice_channel,
        Err(_) => {
            let _ = msg.reply(ctx, "Get in a voice channel idot.").await;
            return Ok(());
        }
    };

    let name: Option<String> = args.single().ok();
    let tag = get_tag(&msg.author);

    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");

    let path = get_theme_path(&tag, &kind, name.as_deref(), storage_client).await?;
    let file_stream = storage_client.get_stream(&path).await?;

    let res = voice::play(ctx, voice_channel.id, voice_channel.guild_id, file_stream).await;

    Ok(())
}
