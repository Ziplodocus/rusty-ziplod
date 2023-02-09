mod commands;
use commands::{ping::PING_COMMAND, zumbor::ZUMBOR_COMMAND};
pub mod config;
use config::Config;
use serenity::{
    framework::{standard::macros::group, StandardFramework},
    prelude::EventHandler,
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

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why)
    }
}
