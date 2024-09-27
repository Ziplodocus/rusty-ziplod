use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::{Context, TypeMapKey},
};

use kalosm::language::*;

use crate::errors::Error;

pub struct ChatBot {
    pub client: Chat,
}

impl ChatBot {
    pub async fn new() -> Result<Self, Error> {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_7b_chat())
            .build()
            .await
            .unwrap();
        let chat = Chat::builder(model)
            .with_system_prompt("You are the almighty discord bot named Ziplod. You're helpful but in a straightforward and uneccessarily rude way. You frequently insult those who are incorrect. You can also be humble in your own strange way.")
            .build();

        Ok(ChatBot { client: chat })
    }
}

impl TypeMapKey for ChatBot {
    type Value = ChatBot;
}

#[command]
pub async fn chat(ctx: &Context, msg: &Message) -> CommandResult {
    println!("Chat triggered {}", msg.content);
    let mut data = ctx.data.write().await;
    let chatbot = data.get_mut::<ChatBot>().take().unwrap();

    let response: String = chatbot.client.add_message(&msg.content).all_text().await;

    dbg!(&response);

    let res = msg.reply(ctx, response).await;

    if let Err(err) = res {
        dbg!(err);
        let _ = msg.reply(ctx, "I'm having a bit of trouble you fool").await;
    }

    Ok(())
}
