mod commands;
mod config;
mod storage;
mod utilities;

use commands::{ping::PING_COMMAND, play::PLAY_COMMAND, zumbor::ZUMBOR_COMMAND};
use config::Config;

use songbird::SerenityInit;

use serenity::{
    framework::{standard::macros::group, StandardFramework},
    model::prelude::UserId,
    prelude::{EventHandler, TypeMapKey},
    Client,
};
use storage::StorageClient;

#[group]
#[commands(ping, zumbor, play)]
struct General;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let config = Config::new();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(config.prefix))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(config.token, config.intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Couldn't create new client!");

    {
        // Make the storage client available to the context
        let mut data = client.data.write().await;

        let storage_client = StorageClient::new().await;

        data.insert::<StorageClient>(storage_client);
    }

    {
        // Create a global list of the running zumbor instances to prevent user from running more than one at once
        let mut data = client.data.write().await;
        data.insert::<ZumborInstances>(ZumborInstances::default())
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why)
    }
}

#[derive(Default, Debug)]
pub struct ZumborInstances {
    instances: Vec<UserId>,
}

impl TypeMapKey for ZumborInstances {
    type Value = ZumborInstances;
}
