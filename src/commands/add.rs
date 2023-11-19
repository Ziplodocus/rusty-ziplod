use bytes::Bytes;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    futures::Stream,
    model::prelude::Message,
    prelude::Context,
};

use crate::{commands::play::count_tracks, errors::Error, storage::StorageClient};

#[command]
pub async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    println!("The add command has been triggered");

    let track_type: Box<str> = match args.single::<String>() {
        Ok(val) => val.into(),
        Err(e) => {
            println!("Failed to determine track type because: {e}");
            msg.reply(
                ctx,
                "You must pass the track type as the first parameter numpty.",
            )
            .await?;
            return Ok(());
        }
    };

    let stream = fetch_attachment_stream(msg).await?;

    let data = ctx.data.read().await;
    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available");
    let num = count_tracks(ctx, &track_type).await.map_err(|o| {
        println!("{:?}", o);
        o
    })?;

    let path: Box<str> = format!("/tracks/{track_type}/{num}").into();

    storage_client.upload_stream(stream, &path, "mp3").await?;

    println!("Add command ended");
    return Ok(());
}

async fn fetch_attachment_stream(
    msg: &Message,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, Error> {
    let file = match msg.attachments.get(0) {
        Some(attach) => attach,
        None => return Err(Error::Plain("That message has no attachments dummy.")),
    };

    let _valid = match &file.content_type {
        Some(val) if val == "mp3" => true,
        _ => return Err(Error::Plain("The attachment is not an mp3")),
    };

    let stream = reqwest::Client::new()
        .get(&file.url)
        .send()
        .await
        .expect("oh well")
        .bytes_stream();

    Ok(stream)
}
