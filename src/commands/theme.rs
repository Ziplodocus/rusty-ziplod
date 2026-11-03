use crate::{
    CommandResult, Context, errors::Error, storage::StorageClient, utilities::random::random_range,
};
use {add::add, delete::delete, list::list, play::play};

mod add;
mod delete;
mod list;
mod play;

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "list", "play", "delete")
)]
pub async fn theme(ctx: Context<'_>) -> CommandResult {
    ctx.say("Try a subcommand dufus").await;
    Ok(())
}

pub fn get_theme_prefix(tag: &str, kind: &str) -> String {
    format!("themes/{}/{}", tag, kind)
}

pub async fn get_theme_path(
    tag: &str,
    kind: &str,
    file_name: Option<&str>,
    client: &StorageClient,
) -> Result<String, Error> {
    match file_name {
        Some(name) => Ok(format!("themes/{}/{}/{}.mp3", tag, kind, name)),
        None => {
            let list = get_theme_list(tag, kind, client).await?;

            let rand: usize = match list.len() {
                0 => return Err(Error::Plain("No themes at all stupid")),
                1 => 0,
                2.. => random_range(0, list.len() - 1),
            };

            let object = list
                .get(rand)
                .take()
                .expect("Previous logic prevents out of bounds index");

            Ok(object.name.clone())
        }
    }
}

pub async fn get_theme_list(
    tag: &str,
    kind: &str,
    client: &StorageClient,
) -> Result<Vec<cloud_storage::Object>, Error> {
    client.get_objects(&get_theme_prefix(tag, kind)).await
}
