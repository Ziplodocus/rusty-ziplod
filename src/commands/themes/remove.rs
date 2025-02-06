use serenity::all::{standard::Args, Context, Message};

use crate::{
    commands::themes::get_tag,
    errors::Error,
    storage::{self, StorageClient},
};

pub async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> Result<(), Error> {
    let kind: String = match args.single() {
        Ok(kind) if (kind == "intro") | (kind == "outro") => kind,
        Ok(_) => {
            let _ = msg
                .reply(
                    ctx,
                    "Delete WHAT you nincompoop? It must be intro or outro!",
                )
                .await;
            return Err(Error::Plain(
                "Delete WHAT you nincompoop? It must be intro or outro!",
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
                .reply(ctx, "Which one to remove you maHOOSIVE idot")
                .await;

            return Err(Error::Plain("Which one to remove you maHOOSIVE idot"));
        }
    };

    let data = ctx.data.read().await;

    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");

    let tag = get_tag(&msg.author);

    let path = format!("themes/{tag}/{kind}/{name}");

    let _ = match storage_client.delete(path.as_str()).await {
        Ok(_) => {
            msg.reply(ctx, "Successfully removed {type} theme {name}")
                .await
        }
        Err(err) => msg.reply(ctx, err.to_string()).await,
    };

    Ok(())
}
