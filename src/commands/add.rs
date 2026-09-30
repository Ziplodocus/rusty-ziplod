use bytes::Bytes;

use serenity::{
    futures::Stream,
    model::channel::{Attachment, Message},
};

use crate::{
    CommandResult, Context, commands::play::count_tracks, errors::Error, storage::StorageClient,
};

#[poise::command(slash_command, prefix_command)]
pub async fn add(
    ctx: Context<'_>,
    msg: Message,
    #[description = "The track type to add"] track_type: String,
) -> CommandResult {
    println!("The add command has been triggered");

    let Some(file) = msg.attachments.first() else {
        ctx.say("That message has no attachments dummy.").await?;
        return Ok(());
    };

    let stream = match fetch_attachment_stream(file.clone()).await {
        Ok(stream) => stream,
        Err(err) => {
            println!("{}", err);
            let _ = ctx.say(err.to_string()).await;
            return Ok(());
        }
    };

    let num = count_tracks(ctx, &track_type).await.map_err(|o| {
        println!("{:?}", o);
        o
    })?;

    let path: String = format!("tracks/{track_type}/{num}.mp3");

    let storage_client = &(ctx.data().storage_client);
    if let Err(_err) = storage_client
        .create_stream(stream, &path, None, "audio/mpeg")
        .await
    {
        ctx.say("Failed to upload your stinky file").await?;
    };

    println!("Add command ended");
    return Ok(());
}

async fn fetch_attachment_stream(
    file: Attachment,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, Error> {
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
