use serenity::{
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::prelude::Message,
    prelude::Context,
    Error,
};

use crate::utilities::{message, random};

#[command]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let voice_channel = message::resolve_voice_channel(ctx, msg).await;

    if let None = voice_channel {
        msg.reply(
            ctx,
            "\n Someone has to be in a voice channel, don't they? idiot.",
        )
        .await;
        return Ok(());
    }

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

    play_track(track_type, track_num).await;

    return Ok(());
}

async fn play_track<'a>(track_type: String, track_num: i32) -> Result<(), Error> {
    Ok(())
}

fn get_random_track_type() -> String {
    "meme".to_owned()
}

async fn count_tracks(track_type: &str) -> i32 {
    return 0;
}
