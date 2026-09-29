mod commands;
mod errors;
mod storage;
mod utilities;
mod voice;

use commands::zumbor::ZumborInstances;
use dotenv::dotenv;
use songbird::serenity::SerenityInit;
use std::env;
use storage::StorageClient;
use poise::serenity_prelude as serenity;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use commands::{
//     add::ADD_COMMAND,
//     list::LIST_COMMAND,
//     ping::PING_COMMAND,
    play::play
//     themes::THEME_COMMAND,
//     zumbor::ZUMBOR_COMMAND,
};

// #[cfg(feature = "chat")]
// use commands::chat::{ChatBot, CHAT_COMMAND};
//
struct Data {
    storage_client: StorageClient,

}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Token");
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let bucket_name = env::var("CLOUD_BUCKET_NAME").expect("Bucket name");
    let prefix = env::var("COMMAND_PREFIX").expect("Prefix");

    println!("Env variables determined.");

    let poise_framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![play()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();


    // Create a new instance of the Client, logging in as a bot.
    let mut serenity =
        serenity::Client::builder(&token, intents)
            .register_songbird()
            .framework(poise_framework)
            .await
            .expect("Err creating client");

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

    // Start listening for events by starting a single shard
    if let Err(why) = serenity.start().await {
        println!("Client error: {why:?}");
    }

}
