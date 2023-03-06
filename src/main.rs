mod commands;
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
    prelude::{EventHandler, TypeMapKey},
    Client,
};

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
        let mut data = client.data.write().await;

        let config_default = GoogClientConfig::default();
        let storage_config = match GoogClientConfig::default().with_auth().await {
            Ok(thing) => thing,
            Err(err) => {
                println!("{:?}", err);
                panic!("{:?}", err);
            }
        };
        let google_client = GoogClient::new(storage_config);
        let storage_client = StorageClient::new(google_client);

        data.insert::<StorageClient>(storage_client);
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why)
    }
}

struct StorageClient {
    client: google_cloud_storage::client::Client,
}

impl StorageClient {
    fn new(client: google_cloud_storage::client::Client) -> Self {
        StorageClient { client }
    }
}

impl TypeMapKey for StorageClient {
    type Value = StorageClient;
}
