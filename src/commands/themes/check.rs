use serenity::all::{standard::Args, Context, Message};

use crate::{errors::Error, storage::StorageClient};

use super::get_tag;

pub async fn check(ctx: &Context, msg: &Message, mut _args: Args) -> Result<(), Error> {
    let data = ctx.data.read().await;

    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");

    // dbg!(&msg.author);

    let intro_path = format!("themes/{}/intro", get_tag(&msg.author));
    let outro_path = format!("themes/{}/outro", get_tag(&msg.author));

    let intros = storage_client.get_objects(intro_path.as_str());
    let outros = storage_client.get_objects(outro_path.as_str());

    let (intros, outros) = tokio::join!(async { intros.await }, async { outros.await });

    let mut reply: String = "Here are your current themes... \nIntros:".to_string();

    let intro_list: Vec<String> = intros
        .unwrap_or(Vec::new())
        .into_iter()
        .map(|theme| theme.name)
        .collect();
    let outro_list: Vec<String> = outros
        .unwrap_or(Vec::new())
        .into_iter()
        .map(|theme| theme.name)
        .collect();

    reply += "\n\t";
    reply += intro_list.join("\n\t").as_str();
    reply += "\nOutros:\n\t";
    reply += outro_list.join("\n\t").as_str();

    msg.reply(ctx, reply).await;

    Ok(())
}
