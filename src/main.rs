mod commands;
mod errors;
mod events;
mod storage;
mod utilities;
mod voice;

#[cfg(feature = "chat")]
use commands::chat::ChatBot;
use commands::{
    add::add,
    list::list,
    ping::ping,
    play::play,
    theme::theme,
    zumbor::{Zumbor, zumbor},
};
use dotenv::dotenv;
use events::Handler;
use poise::{PrefixFrameworkOptions, serenity_prelude as serenity};
use songbird::serenity::SerenityInit;
use std::env;
use storage::StorageClient;

type CommandError = Box<dyn std::error::Error + Send + Sync>;
type CommandResult = Result<(), CommandError>;
type Context<'a> = poise::Context<'a, Data, CommandError>;

pub struct Data {
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
    let _prefix = env::var("COMMAND_PREFIX").expect("Prefix");

    println!("Env variables determined.");

    let data = Data::new().await;

    let prefix_options = PrefixFrameworkOptions {
        prefix: Some("!".into()),
        ..Default::default()
    };

    let poise_framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options,
            commands: vec![play(), zumbor(), add(), ping(), list(), theme()],
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
        .event_handler(Handler)
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
