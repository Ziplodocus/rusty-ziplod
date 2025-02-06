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

    let stream = match fetch_attachment_stream(msg).await {
        Ok(stream) => stream,
        Err(err) => {
            println!("{}", err);
            let _ = msg.reply(ctx, err.to_string()).await;
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available");

    let num = count_tracks(ctx, &track_type).await.map_err(|o| {
        println!("{:?}", o);
        o
    })?;

    let path: String = format!("tracks/{track_type}/{num}.mp3");

    if let Err(_err) = storage_client
        .create_stream(stream, &path, None, "audio/mpeg")
        .await
    {
        println!("Failed to upload the object :(");
    };

    println!("Add command ended");
    return Ok(());
}

async fn fetch_attachment_stream(
    msg: &Message,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, Error> {
    let file = match msg.attachments.first() {
        Some(attach) => attach,
        None => return Err(Error::Plain("That message has no attachments dummy.")),
    };

    let _valid = match &file.content_type {
        Some(val) if val == "audio/mpeg" => true,
        _ => return Err(Error::Plain("The attachment is not an mp3")),
    };

    dbg!(&file.url);

    let stream = reqwest::Client::new()
        .get(&file.url)
        .send()
        .await
        .expect("oh well")
        .bytes_stream();

    Ok(stream)
}
