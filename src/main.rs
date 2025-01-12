mod commands;
mod errors;
mod storage;
mod utilities;
mod voice;

use commands::zumbor::ZumborInstances;
use dotenv::dotenv;
use serenity::all::standard::Configuration;
use serenity::client::{Client, EventHandler};
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::prelude::GatewayIntents;
use songbird::serenity::SerenityInit;
use std::env;
use storage::StorageClient;

use commands::{
    add::ADD_COMMAND, list::LIST_COMMAND, ping::PING_COMMAND, play::PLAY_COMMAND,
    themes::THEME_COMMAND, zumbor::ZUMBOR_COMMAND,
};

#[cfg(feature = "chat")]
use commands::chat::{ChatBot, CHAT_COMMAND};

#[group]
#[commands(ping, zumbor, play, add, list, theme)]
#[cfg_attr(feature = "chat", commands(chat))]
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

    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new().prefix(prefix));

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Couldn't create new client!");

    println!("Client created!");

    {
        // Make storage, chatbot & zumbor client available to the context
        let (mut data, storage_client) =
            tokio::join!(client.data.write(), StorageClient::new(bucket_name),);

        #[cfg(feature = "chat")]
        {
            let chatbot = ChatBot::new().await;

            if let Ok(chatbot) = chatbot {
                data.insert::<ChatBot>(chatbot);
            }

            println!("Chatbot created");
        }

        data.insert::<StorageClient>(storage_client);
        data.insert::<ZumborInstances>(ZumborInstances::default())
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why)
    }
}
