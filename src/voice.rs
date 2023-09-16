use std::{
    io::{stdin, BufRead, BufReader, BufWriter, Write},
    os::windows::prelude::AsHandle,
    process::{Command, Stdio},
    sync::Arc,
    thread,
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
    Call,
};
use tokio::sync::{Mutex, MutexGuard};

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
    file_stream: &[u8],
) -> Result<(), Error> {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let ffmpeg_args = ["-f", "mp3", "-i", "pipe:0", "-f", "f32le", "pipe:1"];

    let mut ffmpeg = Command::new("ffmpeg")
        .args(ffmpeg_args)
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = ffmpeg.stdin.take().unwrap();
    let mut writer = BufWriter::new(stdin);

    let source = Input::new(
        false,
        children_to_reader::<f32>(vec![ffmpeg]),
        Codec::FloatPcm,
        Container::Raw,
        None,
    );

    println!("Writing input...");
    writer.write_all(&file_stream)?;
    drop(writer);
    println!("Written input.");

    let maybe_handler = manager.join(guild_id, channel_id).await;
    let handler_lock = maybe_handler.0;

    println!("Aquiring lock..");
    let mut handler = handler_lock.lock().await;

    println!("Playing source...");

    handler.play_source(source);

    Ok(())
}
