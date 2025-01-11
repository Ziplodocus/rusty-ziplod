use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    futures::Stream,
    model::prelude::{ChannelId, GuildChannel, GuildId, Message},
    prelude::Context,
};

use crate::{
    errors::Error,
    storage::StorageClient,
    utilities::{message, random},
    voice,
};

use cloud_storage::Object;

#[command]
pub async fn list(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("The play command has been triggered");
    let maybe_voice_channel = message::resolve_voice_channel(ctx, msg).await;

    if let Err(err) = maybe_voice_channel {
        msg.reply(ctx, err.to_string()).await?;
        return Ok(());
    }

    let track_type: String = match args.single::<String>() {
        Ok(arg) => arg,
        Err(_) => "meme".to_owned(),
    };

    let all_tracks = get_tracks(ctx, &track_type).await?;

    msg.reply(
        ctx,
        all_tracks
            .into_iter()
            .map(|obj| obj.name)
            .collect::<Vec<String>>()
            .join("\n"),
    )
    .await?;

    // println!("Play command ended");
    return Ok(());
}

pub async fn get_tracks(ctx: &Context, track_type: &str) -> Result<Vec<Object>, Error> {
    let data = ctx.data.read().await;
    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");
    let file_name = format!("tracks/{track_type}/");

    storage_client.get_objects(&file_name).await
}
