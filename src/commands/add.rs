use bytes::Bytes;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::{ChannelId, GuildChannel, GuildId, Message},
    prelude::Context, futures::{Stream, StreamExt},
};

use reqwest::{

};

use crate::{
    commands::play::count_tracks,
    errors::Error,
    storage::{StorageClient, self},
    utilities::{message, random},
};

#[command]
pub async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("The add command has been triggered");

    let track_type: String = args.single::<String>()?;

    let track_count = count_tracks(ctx, &track_type)
        .await?
        .try_into()
        .unwrap_or(0);

    let stream = fetch_attachment_stream(msg).await?;


    let data = ctx.data.read().await;
    let storage_client = data.get::<StorageClient>().unwrap();
    let num = count_tracks(ctx, &track_type).await.map_err(|o| {
        println!("{:?}", o);
        Error::Plain("Error counting tracks")
    })?;

    let path = format!("/tracks/{track_type}/{num}");

    storage_client.upload_stream(stream, &path, "mp3").await?;

    println!("Add command ended");
    return Ok(());
}


async fn fetch_attachment_stream(msg: &Message) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, Error>
{
    let file = match msg.attachments.get(0) {
        Some(attach) => attach,
        None => return Err(Error::Plain("That message has no attachments dummy."))
    };

    let _valid = match &file.content_type {
        Some(val) if val == "mp3" => true,
        _ => return Err(Error::Plain("The attachment is not an mp3")),
    };

    let stream = reqwest::Client::new()
        .get(&file.url)
        .send()
        .await.expect("oh well")
        .bytes_stream();

        Ok(stream)
}
