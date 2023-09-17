use std::{
    collections::HashMap,
    io::{stdin, BufRead, BufReader, BufWriter, Write},
    os::windows::prelude::AsHandle,
    process::{Command, Stdio},
    sync::Arc,
    thread, rc::Rc,
};

use serde_json::{Map, Value};
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

use crate::{errors::Error, audio_conversion};

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
    file_stream: Vec<u8>,
) -> Result<(), Error> {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let (ffmpeg, meta, writer_handle) = audio_conversion::convert(file_stream.into(), "f32le")?;

    let source = Input::new(
        meta.is_stereo,
        children_to_reader::<f32>(vec![ffmpeg]),
        Codec::FloatPcm,
        Container::Raw,
        None,
    );

    let maybe_handler = manager.join(guild_id, channel_id).await;
    let handler_lock = maybe_handler.0;
    let mut handler = handler_lock.lock().await;
    println!("Reading/Playing output of conversion...");
    handler.play_source(source);

    Ok(())
}