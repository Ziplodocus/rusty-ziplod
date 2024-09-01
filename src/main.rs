mod assets;
mod audio_conversion;
mod commands;
mod errors;
mod storage;
mod utilities;
mod voice;

use dotenv::dotenv;
use std::env;

use commands::{add::ADD_COMMAND, ping::PING_COMMAND, play::PLAY_COMMAND, zumbor::ZUMBOR_COMMAND};

// Import the `Context` to handle commands.

use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::prelude::GatewayIntents;
use serenity::{
    client::{Client, EventHandler},
    model::prelude::UserId,
    prelude::TypeMapKey,
};

use songbird::serenity::SerenityInit;

use storage::StorageClient;

#[group]
#[commands(ping, zumbor, play, add)]
struct General;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let bucket_name = env::var("CLOUD_BUCKET_NAME").expect("Bucket name");
    let prefix = env::var("COMMAND_PREFIX").expect("Prefix");

    println!("Env variables determined.");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(prefix))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Couldn't create new client!");

    println!("Client creatd!");

    {
        // Make the storage client available to the context
        let mut data = client.data.write().await;

        let storage_client = StorageClient::new(bucket_name).await;

        // add_stereo_meta_information(&storage_client).await;

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
