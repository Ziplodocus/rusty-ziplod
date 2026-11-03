use bytes::Bytes;

use serenity::{
    futures::Stream,
    model::channel::{Attachment, Message},
};

use crate::{CommandError, CommandResult, Context, commands::play::count_tracks, errors::Error};

#[poise::command(slash_command, prefix_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "MP3 file to upload"] file: Attachment,
    #[description = "The track type to add"] track_type: String,
) -> CommandResult {
    let _ = ctx.defer_ephemeral().await;

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
    if let Err(err) = storage_client
        .create_stream(stream, &path, None, "audio/mpeg")
        .await
    {
        ctx.say("Failed to upload your stinky file").await?;
        return Err(err.into());
    };

    ctx.say(format!("Uploaded as {track_type}/{num}.mp3"));

    Ok(())
}

async fn fetch_attachment_stream(
    file: Attachment,
) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, Error> {
    let _valid = match &file.content_type {
        Some(val) if val == "audio/mpeg" => true,
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
