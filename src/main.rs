mod commands;
mod errors;
mod storage;
mod utilities;
mod voice;
use commands::chat::ChatBot;
use commands::zumbor::ZumborInstances;
use commands::{
    add::ADD_COMMAND, chat::CHAT_COMMAND, ping::PING_COMMAND, play::PLAY_COMMAND,
    zumbor::ZUMBOR_COMMAND,
};
use dotenv::dotenv;
use serenity::all::standard::Configuration;
use serenity::client::{Client, EventHandler};
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::prelude::GatewayIntents;
use songbird::serenity::SerenityInit;
use std::env;
use storage::StorageClient;

#[group]
#[commands(ping, zumbor, play, add, chat)]
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
        let (mut data, storage_client, chatbot) = tokio::join!(
            client.data.write(),
            StorageClient::new(bucket_name),
            ChatBot::new()
        );

        if let Ok(chatbot) = chatbot {
            data.insert::<ChatBot>(chatbot);
        }
        data.insert::<StorageClient>(storage_client);
        data.insert::<ZumborInstances>(ZumborInstances::default())
    }

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why)
    }
}
