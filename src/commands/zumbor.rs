use serenity::{all::UserId, model::prelude::Message};

mod attributes;
mod effects;
mod encounter;
mod initialise;
mod player;
mod ui;
use initialise::start;
use tokio::sync::RwLock;

use crate::{CommandResult, Context, errors::Error};

pub struct Zumbor {
    instances: RwLock<ZumborInstances>,
}

impl Zumbor {
    pub fn new() -> Self {
        let instances = RwLock::new(ZumborInstances::default());

        Zumbor { instances }
    }
}

#[poise::command(slash_command, prefix_command)]
pub async fn zumbor(ctx: Context<'_>) -> CommandResult {
    let res = start(ctx).await;
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
