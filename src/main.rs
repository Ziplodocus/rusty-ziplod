mod commands;
mod errors;
mod storage;
mod utilities;
mod voice;

use dotenv::dotenv;
use poise::{Prefix, PrefixFrameworkOptions, serenity_prelude as serenity};
use rustls::crypto::CryptoProvider;
use songbird::serenity::SerenityInit;
use std::env;
use storage::StorageClient;
use tokio::sync::RwLock;

type CommandError = Box<dyn std::error::Error + Send + Sync>;
type CommandResult = Result<(), CommandError>;
type Context<'a> = poise::Context<'a, Data, CommandError>;

use commands::{
    add::add,
    list::list,
    ping::ping,
    play::play,
    // themes::themes,
    zumbor::{Zumbor, zumbor},
};

#[cfg(feature = "chat")]
use crate::commands::chat::ChatBot;

struct Data {
    storage_client: StorageClient,
    zumbor: Zumbor,
    #[cfg(feature = "chat")]
    chat: ChatBot,
}

impl Data {
    pub async fn new() -> Self {
        let bucket_name = env::var("CLOUD_BUCKET_NAME").expect("Bucket name");

        let storage_client = StorageClient::new(bucket_name).await;
        let zumbor = Zumbor::new();

        #[cfg(feature = "chat")]
        let chat = ChatBot::new().await.expect("ChatBot to be created");

        Data {
            storage_client,
            zumbor,
            #[cfg(feature = "chat")]
            chat,
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let token = env::var("DISCORD_TOKEN").expect("Token");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let prefix = env::var("COMMAND_PREFIX").expect("Prefix");

    println!("Env variables determined.");

    // let crypto_provider = CryptoProvider::get_default().unwrap();
    // crypto_provider..install_default();

    let data = Data::new().await;

    let mut prefix_options = PrefixFrameworkOptions::default();
    prefix_options.prefix = Some("!".into());

    let poise_framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options,
            commands: vec![play(), zumbor(), add(), ping(), list()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    // Create a new instance of the Client, logging in as a bot.
    let mut serenity = serenity::Client::builder(&token, intents)
        .register_songbird()
        .framework(poise_framework)
        .await
        .expect("Err creating client");

    println!("Client created!");

    // Start listening for events by starting a single shard
    if let Err(why) = serenity.start().await {
        println!("Client error: {why:?}");
    }
}
