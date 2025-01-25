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

use crate::errors::Error;

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

impl ZumborInstances {
    pub fn add(&mut self, user_id: UserId) -> Result<(), Error> {
        if self.instances.contains(&user_id) {
            Err(Error::Plain(
                "The user currently has an active Zumbor instance",
            ))
        } else {
            self.instances.push(user_id);
            println!("{:?}", self.instances);
            Ok(())
        }
    }

    pub fn remove(&mut self, user_id: UserId) {
        println!("{:?}", self.instances);
        self.instances
            .retain(|&instance_user_id| instance_user_id != user_id);
        println!("{:?}", self.instances);
    }
}

impl TypeMapKey for ZumborInstances {
    type Value = ZumborInstances;
}
