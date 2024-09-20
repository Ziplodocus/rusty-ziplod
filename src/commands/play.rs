use std::sync::Arc;

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

#[command]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("The play command has been triggered");
    let maybe_voice_channel = message::resolve_voice_channel(ctx, msg).await;

    if let Err(err) = maybe_voice_channel {
        msg.reply(ctx, err.to_string()).await?;
        return Ok(());
    }

    let voice_channel = maybe_voice_channel?;

    let track_type: String = match args.single::<String>() {
        Ok(arg) => arg,
        Err(_) => get_random_track_type(),
    };

    let track_count: u32 = match count_tracks(ctx, &track_type).await {
        Ok(val) => val,
        Err(e) => {
            msg.reply(
                ctx,
                "The request can't be completed right now dufus.".to_string(),
            )
            .await?;
            println!("{e}");
            return Ok(());
        }
    }
    .try_into()
    .unwrap_or(1);

    let track_num = match track_count {
        0...1 => 0,
        _ => args
            .single::<u32>()
            .unwrap_or_else(|_| random::random_range(0, track_count - 1)),
    };

    if track_num >= track_count {
        msg.reply(ctx, format!("There is no {track_type} {track_num}"))
            .await?;
        return Ok(());
    }

    play_track(ctx, track_type, track_num, voice_channel)
        .await
        .map_err(|o| {
            println!("{o}");
            format!("{o}")
        })?;

    println!("Play command ended");
    return Ok(());
}

async fn play_track<'a>(
    ctx: &Context,
    track_type: String,
    track_num: u32,
    voice_channel: GuildChannel,
) -> Result<(), Error> {
    println!("Fetching track...");

    let (track_stream, is_stereo) = fetch_track(ctx, &track_type, track_num).await?;

    if is_stereo.is_none() {
        return Err(Error::Plain(
            "File doesn't have stereo meta data associated with it!",
        ));
    }

    let guild_id = voice_channel
        .guild(ctx)
        .expect("The channel to be in a guild")
        .id;

    play_audio_in_channel(
        ctx,
        track_stream,
        voice_channel.id,
        guild_id,
        is_stereo.unwrap(),
    )
    .await
    .map_err(|o| {
        println!("{o}");
        o
    })?;

    Ok(())
}

fn get_random_track_type() -> String {
    // 69
    "meme".to_owned()
}

pub async fn count_tracks(ctx: &Context, track_type: &str) -> Result<usize, Error> {
    let data = ctx.data.read().await;
    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");
    let file_name = format!("tracks/{track_type}/");
    storage_client.get_count(&file_name).await
}

async fn fetch_track<'a>(
    ctx: &Context,
    track_type: &str,
    track_num: u32,
) -> Result<(impl Stream<Item = Result<u8, Error>> + Unpin, Option<bool>), Error> {
    let data = ctx.data.read().await;
    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");
    println!("Fetching {track_type} {track_num}");
    let file_name: Arc<str> = format!("tracks/{track_type}/{track_num}.mp3").into();

    tokio::try_join!(
        storage_client.get_stream(&file_name),
        storage_client.is_stereo(&file_name)
    )
}

async fn play_audio_in_channel(
    ctx: &Context,
    audio_stream: impl Stream<Item = Result<u8, Error>> + Unpin,
    channel: ChannelId,
    guild: GuildId,
    is_stereo: bool,
) -> Result<(), Error> {
    println!("Streaming audio to channel {channel}...");
    voice::play(ctx, channel, guild, audio_stream, is_stereo).await?;

    Ok(())
}
