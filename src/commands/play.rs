use std::mem::MaybeUninit;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::{ChannelId, GuildChannel, Message},
    prelude::Context,
};

use crate::{
    errors::Error,
    storage::StorageClient,
    utilities::{message, random},
};

#[command]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("The play command has been triggered");
    let maybe_voice_channel = message::resolve_voice_channel(ctx, msg).await;

    if let Err(err) = maybe_voice_channel {
        msg.reply(ctx, err).await?;
        return Ok(());
    }

    let voice_channel = maybe_voice_channel.unwrap();

    let track_type: String = match args.single::<String>() {
        Ok(arg) => arg.into(),
        Err(_) => get_random_track_type(),
    };

    let track_count = count_tracks(track_type.as_str()).await;

    let mut track_num = args
        .single::<i32>()
        .unwrap_or_else(|_| random::random_range(0, track_count));

    if track_num > track_count {
        track_num = random::random_range(0, track_count);
    }

    play_track(ctx, track_type, track_num, voice_channel)
        .await
        .map_err(|o| format!("{o}"));

    return Ok(());
}

async fn play_track<'a>(
    ctx: &Context,
    track_type: String,
    track_num: i32,
    voice_channel: GuildChannel,
) -> Result<(), Error> {
    println!("Fecthing track...");
    let track_stream = fetch_track(ctx, track_type, track_num).await?;
    println!("Playing track...");
    play_audio_from_stream_in_channel(track_stream, voice_channel.id).await?;
    Ok(())
}

fn get_random_track_type() -> String {
    "meme".to_owned()
}

async fn count_tracks(_track_type: &str) -> i32 {
    return 1;
}

async fn fetch_track(ctx: &Context, track_type: String, track_num: i32) -> Result<Vec<u8>, Error> {
    let data = ctx.data.read().await;
    let storage_client = data.get::<StorageClient>().unwrap();
    storage_client
        .download(&"tracks/{track_type}/{track_num}.mp3".into())
        .await
}

async fn play_audio_from_stream_in_channel(
    audio_stream: Vec<u8>,
    channel: ChannelId,
) -> Result<(), Error> {
    println!("Playing audio in channel {channel}");
    Ok(())
}
