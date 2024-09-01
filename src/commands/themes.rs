use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

use crate::errors::Error;

#[command]
pub async fn theme(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let subcommand: String = args.single()?;

    let res: Result<&str, Error> = match subcommand.as_str() {
        // "add" => add(ctx,msg,args),
        // "check" => check(ctx, msg, args),
        // "play" => play(ctx,msg, args),
        // "remove" => remove(ctx, msg, args),
        _ => Err(Error::Plain("No matching subcommand")),
    };

    match res {
        Ok(some) => println!("{}", some),
        Err(err) => println!("{}", err),
    }

    Ok(())
}
