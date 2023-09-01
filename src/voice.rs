use serenity::{
    framework::standard::{macros::command, CommandResult, CommonOptions},
    futures::channel::oneshot::channel,
    model::prelude::{ChannelId, GuildId, Message},
    prelude::Context,
};
use songbird::input::{codec::OpusDecoderState, Codec, Container, Input, Reader};

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
    file: Vec<u8>,
) -> Result<(), Error> {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    println!("Got the voice manager.");

    let source = Input::new(
        true,
        Reader::from_memory(file),
        Codec::Pcm,
        Container::Raw,
        None,
    );

    println!("Got the source.");

    let maybe_handler = manager.join(guild_id, channel_id).await;

    let handler_lock = maybe_handler.0;

    println!("Got the handler lock.");

    let mut handler = handler_lock.lock().await;

    println!("Got the handler.");

    handler.play_source(source);

    println!("Played/Playing the source.");

    Ok(())
}
