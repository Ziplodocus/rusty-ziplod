use serenity::all::{standard::Args, Context, Message};

use crate::{commands::themes::get_tag, errors::Error, storage::StorageClient};

pub async fn add(ctx: &Context, msg: &Message, mut args: Args) -> Result<(), Error> {
    let kind: String = match args.single() {
        Ok(kind) if (kind == "intro") | (kind == "outro") => kind,
        Ok(_) => {
            let _ = msg
                .reply(
                    ctx,
                    "Register as WHAT you neanderthal? It must be intro or outro!",
                )
                .await;
            return Err(Error::Plain(
                "Register as WHAT you neanderthal? It must be intro or outro!",
            ));
        }
        Err(_) => {
            let _ = msg.reply(ctx, "Specify intro or outro you twit").await;
            return Err(Error::Plain("Specify intro or outro you twit"));
        }
    };

    let name: String = match args.single() {
        Ok(name) => name,
        Err(_) => {
            let _ = msg
                .reply(
                    ctx,
                    "The theme must have a (one word) name! I recommend 'moron'",
                )
                .await;

            return Err(Error::Plain(
                "The theme must have a (one word) name! I recommend 'moron'",
            ));
        }
    };
    let attachment = match msg.attachments.first() {
        Some(attach) => attach,
        None => {
            let _ = msg.reply(ctx, "You must attach an mp3 file dufus").await;
            return Err(Error::Plain("You must attach an mp3 file dufus"));
        }
    };

    dbg!(attachment.content_type.as_ref());
    let content_type = match attachment.content_type.as_deref() {
        Some("audio/mpeg") => kind.to_owned(),
        _ => {
            let _ = msg
                .reply(
                    ctx,
                    "MP3 FILE. Everything else is garbage. Like your mother.",
                )
                .await;
            return Ok(());
        }
    };

    if attachment.size > 3347520 {
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

    let path = format!("themes/{}/{}/{}.mp3", get_tag(&msg.author), kind, name);

    let res = storage_client
        .create_stream(stream, &path, None, &content_type)
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
