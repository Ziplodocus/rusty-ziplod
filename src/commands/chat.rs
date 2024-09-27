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
        let model = Llama::new_chat().await.unwrap();
        let chat = Chat::builder(model)
            .with_system_prompt("You are an almighty discord bot, and you know it. You're helpful but in an uneccessarily rude way and you have no patience for incoorectness.")
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
    {
        chatbot;
    }

    let res = msg.reply(ctx, response).await;

    dbg!(res);

    Ok(())
}
