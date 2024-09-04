use futures_util::{Stream, StreamExt};
use std::{
    io::Write,
    process::{Command, Stdio},
};

use serenity::{
    model::prelude::{ChannelId, GuildId},
    prelude::Context,
};
use songbird::input::{children_to_reader, Codec, Container, Input};

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
    mut file_stream: impl Stream<Item = Result<u8, Error>> + Unpin,
    is_stereo: bool,
) -> Result<(), Error> {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let ffmpeg_args = [
        "-v", "error", // "-f",
        // &meta.format_name,
        "-i", "pipe:0", "-f", "f32le", "pipe:1",
    ];

    let mut ffmpeg = Command::new("ffmpeg")
        .args(ffmpeg_args)
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    // Ensure we have a handle to ffmpeg's stdin.
    let mut stdin = ffmpeg.stdin.take().expect("Failed to open stdin");

    let source = Input::new(
        is_stereo,
        children_to_reader::<f32>(vec![ffmpeg]),
        Codec::FloatPcm,
        Container::Raw,
        None,
    );

    {
        let maybe_handler = manager.join(guild_id, channel_id).await;
        let handler_lock = maybe_handler.0;
        let mut handler = handler_lock.lock().await;
        handler.stop();
        handler.play_source(source);
        println!("Started playing...");
    }

    // Spawn an async task to read from the stream and send data to the blocking thread
    let mut buffer = Vec::with_capacity(1024);

    while let Some(byte) = file_stream.next().await {
        buffer.push(byte.unwrap());

        if buffer.len() >= 1024 {
            stdin.write_all(&buffer)?;
            buffer.clear();
        }
    }

    // Send any remaining bytes in the buffer
    if !buffer.is_empty() {
        stdin.write_all(&buffer)?;
    }

    Ok(())
}
