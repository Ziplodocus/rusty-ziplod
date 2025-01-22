use serenity::all::{standard::Args, Context, Message, User};

use crate::{errors::Error, storage::StorageClient, utilities::random::random_range};

use super::get_tag;

pub async fn check(ctx: &Context, msg: &Message, mut _args: Args) -> Result<(), Error> {
    let data = ctx.data.read().await;

    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in the context");

    // dbg!(&msg.author);

    let intro_path = format!("themes/{}/intro", get_tag(&msg.author));
    let outro_path = format!("themes/{}/outro", get_tag(&msg.author));

    let intros = storage_client.get_objects(&intro_path);
    let outros = storage_client.get_objects(&outro_path);

    let (intros, outros) = tokio::join!(intros, outros);

    let mut reply: String = "Here are your current themes... \nIntros:".to_string();

    let intro_list: Vec<String> = intros
        .unwrap_or(Vec::new())
        .into_iter()
        .map(|theme| theme.name.replace(&intro_path, ""))
        .collect();
    let outro_list: Vec<String> = outros
        .unwrap_or(Vec::new())
        .into_iter()
        .map(|theme| theme.name.replace(&outro_path, ""))
        .collect();

    reply += "\n\t";
    reply += intro_list.join("\n\t").as_str();
    reply += "\nOutros:\n\t";
    reply += outro_list.join("\n\t").as_str();

    msg.reply(ctx, reply).await;

    Ok(())
}

pub fn get_theme_prefix(tag: &str, kind: &str) -> String {
    format!("themes/{}/{}", tag, kind)
}

pub async fn get_theme_path(
    tag: &str,
    kind: &str,
    file_name: Option<&str>,
    client: StorageClient,
) -> Result<String, Error> {
    match file_name {
        Some(name) => Ok(format!("themes/{}/{}/{}", tag, kind, name)),
        None => {
            let list = get_theme_list(tag, kind, client).await?;
            let rand: usize = random_range(0, list.len() - 1);
            let object = list
                .get(rand)
                .take()
                .expect("Random number is within the vector indices");
            Ok(object.name.clone())
        }
    }
}

pub async fn get_theme_list(
    tag: &str,
    kind: &str,
    client: StorageClient,
) -> Result<Vec<cloud_storage::Object>, Error> {
    let path = format!("themes/{}/{}", tag, kind);
    client.get_objects(&get_theme_prefix(tag, kind)).await
}
