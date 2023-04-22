mod commands;
mod storage;
use std::env;

use commands::{ping::PING_COMMAND, zumbor::ZUMBOR_COMMAND};
pub mod config;
use config::Config;
use google_cloud_default::WithAuthExt;
use google_cloud_storage::client::{Client as GoogClient, ClientConfig as GoogClientConfig};
use serde_json::json;
use serenity::{
    framework::{standard::macros::group, StandardFramework},
    futures::lock::Mutex,
    model::prelude::UserId,
    prelude::{EventHandler, TypeMapKey},
    Client,
};
use storage::StorageClient;

#[group]
#[commands(ping, zumbor)]
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

#[derive(Default)]
pub struct ZumborInstances {
    instances: Vec<UserId>,
}

impl TypeMapKey for ZumborInstances {
    type Value = ZumborInstances;
}
