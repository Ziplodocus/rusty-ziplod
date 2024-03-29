use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

mod effects;
mod encounter;
mod initialise;
pub mod player;
mod ui;
use initialise::start;

#[command]
pub async fn zumbor(ctx: &Context, msg: &Message) -> CommandResult {
    let res = start(ctx, msg).await;
    match res {
        Ok(some) => println!("{}", some),
        Err(err) => println!("{}", err),
    }
    Ok(())
}
