use crate::{errors::Error, storage::StorageClient, utilities::random::random_range};
use serenity::{
    all::User,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

mod add;
mod check;
mod play;

#[command]
pub async fn theme(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let subcommand = args.single::<String>().map_err(|err| {
        println!("{:?}", err);
        return Error::Plain("Uh oh it went wrong");
    })?;

    let res: Result<(), Error> = match subcommand.as_str() {
        "add" => add::add(ctx, msg, args).await,
        "check" => check::check(ctx, msg, args).await,
        "play" => play::play(ctx, msg, args).await,
        // "remove" => remove(ctx, msg, args),
        _ => Err(Error::Plain("No matching subcommand")),
    };

    if let Err(e) = res {
        println!("{}", e);
    }

    Ok(())
}

pub fn get_tag(user: &User) -> String {
    let maybe_global_name = user.global_name.clone();
    return maybe_global_name.unwrap_or(user.tag());
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
        Some(name) => Ok(format!("themes/{}/{}/{}", tag, kind, name)),
        None => {
            let list = get_theme_list(tag, kind, &client).await?;
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
    client: &StorageClient,
) -> Result<Vec<cloud_storage::Object>, Error> {
    client.get_objects(&get_theme_prefix(tag, kind)).await
}
