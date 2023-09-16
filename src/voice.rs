use std::{
    io::Write,
    process::{Command, Stdio},
};

use serenity::{
    framework::standard::{macros::command, CommandResult, CommonOptions},
    futures::{channel::oneshot::channel, Stream, StreamExt},
    model::prelude::{ChannelId, GuildId, Message},
    prelude::Context,
};
use songbird::{
    input::{
        children_to_reader, codec::OpusDecoderState, ChildContainer, Codec, Container, Input,
        Reader,
    },
    tracks::Track,
};

use crate::errors::Error;

// async fn join(ctx: &Context, channel_id: ChannelId, guild_id: GuildId) -> CommandResult {
//     let manager = songbird::get(ctx)
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();

//     let _handler = manager.join(guild_id, channel_id).await;

//     Ok(())
// }

// async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
//     let guild = msg.guild(&ctx.cache).unwrap();
//     let guild_id = guild.id;

//     let manager = songbird::get(ctx)
//         .await
//         .expect("Songbird Voice client placed in at initialisation.")
//         .clone();
//     let has_handler = manager.get(guild_id).is_some();

//     if has_handler {
//         if let Err(e) = manager.remove(guild_id).await {
//             check_msg(
//                 msg.channel_id
//                     .say(&ctx.http, format!("Failed: {:?}", e))
//                     .await,
//             );
//         }

//         check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
//     } else {
//         check_msg(msg.reply(ctx, "Not in a voice channel").await);
//     }

//     Ok(())
// }

pub async fn play(
    ctx: &Context,
    channel_id: ChannelId,
    guild_id: GuildId,
    mut file_stream: impl Stream<Item = Result<u8, cloud_storage::Error>> + Unpin,
) -> Result<(), Error> {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let ffmpeg_args = ["-f", "mp3", "-i", "pipe:", "-f", "pcm_f32le", "-"];

    let mut ffmpeg = Command::new("ffmpeg")
        .args(ffmpeg_args)
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = ffmpeg.stdin.take().unwrap();

    while let Some(item) = file_stream.next().await {
        let item = item?;
        stdin.write_all(&[item])?;
    }

    drop(stdin);

    let source = Input::new(
        true,
        Reader::from(ffmpeg),
        Codec::FloatPcm,
        Container::Raw,
        None,
    );

    let maybe_handler = manager.join(guild_id, channel_id).await;

    let handler_lock = maybe_handler.0;

    let mut handler = handler_lock.lock().await;

    let hand = handler.play_source(source);

    match hand.play() {
        Ok(_) => println!("Play success."),
        Err(err) => println!("{err}"),
    };

    println!("Played/Playing the source.");

    Ok(())
}
