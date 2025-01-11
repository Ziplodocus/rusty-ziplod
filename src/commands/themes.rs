use crate::errors::Error;
use serenity::{
    all::User,
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

mod check;

#[command]
pub async fn theme(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let subcommand = args.single::<String>().map_err(|err| {
        println!("{:?}", err);
        return Error::Plain("Uh oh it went wrong");
    })?;

    let res: Result<(), Error> = match subcommand.as_str() {
        // "add" => add(ctx,msg,args),
        "check" => check::check(ctx, msg, args).await,
        // "play" => play(ctx,msg, args),
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
