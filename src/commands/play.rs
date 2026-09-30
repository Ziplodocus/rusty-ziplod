use serenity::{
    futures::Stream,
    model::{
        guild::Member,
        prelude::{GuildChannel, Message},
    },
};

use crate::{
    CommandResult, Context,
    errors::Error,
    utilities::{message, random},
    voice,
};

#[poise::command(slash_command, prefix_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "The type of track to play"] track_type: Option<String>,
    #[description = "Track number to play"] track_num: Option<u32>,
    #[description = "The user to play to"] member: Option<Member>,
) -> CommandResult {
    println!("The play command has been triggered");

    let member = match member {
        Some(member) => member,
        None => ctx
            .author_member()
            .await
            .expect("Author is member")
            .into_owned(),
    };

    ctx.guild_id().unwrap().channels(ctx);

    let voice_res = message::fetch_voice_channel(ctx, &member).await;
    let Ok(voice_channel) = voice_res else {
        let _res = ctx.say(voice_res.err().unwrap().to_string()).await;
        return Ok(());
    };

    let track_type: String = track_type.unwrap_or_else(get_random_track_type);

    let track_count: u32 = match count_tracks(ctx, &track_type).await {
        Ok(val) => val,
        Err(e) => {
            ctx.say("The request can't be completed right now dufus.")
                .await?;
            println!("{e}");
            return Ok(());
        }
    }
    .try_into()
    .unwrap_or(1);

    let track_num = match track_count {
        0...1 => 0,
        _ => track_num.unwrap_or_else(|| random::random_range(0, track_count - 1)),
    };

    if track_num >= track_count {
        ctx.say(format!("There is no {track_type} {track_num}"))
            .await?;
        return Ok(());
    }

    play_track(ctx, track_type, track_num, voice_channel)
        .await
        .map_err(|o| {
            println!("{o}");
            format!("{o}")
        })?;

    // println!("Play command ended");
    return Ok(());
}

async fn play_track(
    ctx: Context<'_>,
    track_type: String,
    track_num: u32,
    voice_channel: GuildChannel,
) -> Result<(), Error> {
    println!("Fetching track...");

    let track_stream = fetch_track(ctx, &track_type, track_num).await?;

    let guild_id = voice_channel
        .guild(&ctx)
        .expect("The channel to be in a guild")
        .id;

    voice::play(ctx, voice_channel.id, guild_id, track_stream)
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

pub async fn count_tracks(ctx: Context<'_>, track_type: &str) -> Result<usize, Error> {
    let storage_client = &(ctx.data().storage_client);
    let file_name = format!("tracks/{track_type}/");
    storage_client.get_count(&file_name).await
}

async fn fetch_track<'a>(
    ctx: Context<'_>,
    track_type: &'a str,
    track_num: u32,
) -> Result<impl Stream<Item = Result<u8, Error>> + Unpin, Error> {
    let storage_client = &(ctx.data().storage_client);
    println!("Fetching {track_type} {track_num}");
    let file_name = format!("tracks/{track_type}/{track_num}.mp3");

    storage_client.get_stream(file_name).await
}
