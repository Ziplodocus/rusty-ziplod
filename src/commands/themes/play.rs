use serenity::all::{standard::Args, Context, Message};

use crate::{errors::Error, storage::StorageClient, utilities::message::resolve_voice_channel};

use super::get_tag;

pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> Result<(), Error> {
    let data = ctx.data.read().await;

    let kind: Box<str> = match args.single::<String>() {
        Ok(kind) if kind == "intro" || kind == "outro" => kind.into(),
        _ => {
            msg.reply(ctx, "Try specifying intro or outro dimwit").await;
            return Ok(());
        }
    };

    let voice_channel = match resolve_voice_channel(ctx, msg).await {
        Ok(voice_channel) => voice_channel,
        Err(_) => {
            msg.reply(ctx, "Get in a voice channel idot.").await;
            return Ok(());
        }
    };

    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");

    voice::play(voiceState, msg.args[0], msg.args[1]);

    Ok(())
}
