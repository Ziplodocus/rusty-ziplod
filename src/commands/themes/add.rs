use serenity::all::{
    standard::{Args, CommandResult},
    Context, Message,
};

use crate::{
    commands::themes::{get_tag, get_theme_path},
    errors::Error,
    storage::StorageClient,
};

pub async fn add(ctx: &Context, msg: &Message, mut args: Args) -> Result<(), Error> {
    let attachment = msg
        .attachments
        .first()
        .ok_or(Error::Plain("You must attach an mp3 file dufus"))?;
    let kind: String = args
        .single()
        .or(Err(Error::Plain("Specify intro or outro you twit")))?;
    let name: String = args.single().or(Err(Error::Plain(
        "The theme must have a (one word) name! I recommend 'moron'",
    )))?;

    // Validation of user inputs
    if kind != "intro" && kind != "outro" {
        let _ = msg
            .reply(
                ctx,
                "Register as WHAT you neanderthal? It must be intro or outro!",
            )
            .await;
    }

    let content_type = attachment.content_type.as_ref();

    if content_type.is_some_and(|val| val == "audio/mpeg") {
        let _ = msg
            .reply(
                ctx,
                "MP3 FILE. Everything else is garbage. Like your mother.",
            )
            .await;
        return Ok(());
    }

    if attachment.size > 347520 {
        let _ = msg
            .reply(
                ctx,
                "No one wants to hear your life story. Keep it short and sweet.",
            )
            .await;
        return Ok(());
    }

    let stream = reqwest::Client::new()
        .get(&attachment.url)
        .send()
        .await
        .expect("oh well")
        .bytes_stream();

    let data = ctx.data.read().await;

    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");

    let path = format!("themes/{}/{}/{}", get_tag(&msg.author), kind, name);

    let res = storage_client
        .create_stream(stream, &path, None, "audio/mpeg")
        .await;

    let _ = match res {
        Ok(item) => {
            dbg!(item);
            msg.reply(ctx, format!("Successfully registered {} {}", kind, name))
                .await
        }
        Err(err) => {
            dbg!(err);
            msg.reply(ctx, format!("Unsuccessfully registered {} {}", kind, name))
                .await
        }
    };

    Ok(())
}
