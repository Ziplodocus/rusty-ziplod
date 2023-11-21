use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
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
        msg.reply(ctx, err).await?;
        return Ok(());
    }

    let voice_channel = maybe_voice_channel?;

    let track_type: String = match args.single::<String>() {
        Ok(arg) => arg,
        Err(_) => get_random_track_type(),
    };

    let mut track_count: u32 = match count_tracks(ctx, &track_type).await {
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

    let track_stream = fetch_track(ctx, &track_type, track_num).await?;

    let guild_id = voice_channel
        .guild(ctx)
        .expect("The channel to be in a guild")
        .id;

    play_audio_in_channel(ctx, track_stream, voice_channel.id, guild_id)
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
    storage_client.fetch_count(&file_name).await
}

async fn fetch_track(ctx: &Context, track_type: &str, track_num: u32) -> Result<Vec<u8>, Error> {
    let data = ctx.data.read().await;
    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");
    println!("Fetching {track_type} {track_num}");
    let file_name = format!("tracks/{track_type}/{track_num}.mp3");

    storage_client.download(&file_name).await
}

async fn play_audio_in_channel(
    ctx: &Context,
    audio_stream: Vec<u8>,
    channel: ChannelId,
    guild: GuildId,
) -> Result<(), Error> {
    println!("Streaming audio to channel {channel}...");
    voice::play(ctx, channel, guild, audio_stream).await?;

    Ok(())
}
