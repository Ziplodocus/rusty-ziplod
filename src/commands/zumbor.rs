use serenity::{
    all::UserId,
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::{Context, TypeMapKey},
};

mod attributes;
mod effects;
mod encounter;
mod initialise;
mod player;
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

#[derive(Default, Debug)]
pub struct ZumborInstances {
    instances: Vec<UserId>,
}

impl TypeMapKey for ZumborInstances {
    type Value = ZumborInstances;
}
