use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::{Context, TypeMapKey},
};

use kalosm::{
    language::{Chat, Llama, LlamaSource},
    sound::TextStream,
};

use crate::errors::Error;

pub struct ChatBot {
    pub client: Chat,
}

impl ChatBot {
    pub async fn new() -> Result<Self, Error> {
        let model = Llama::builder()
            .with_source(LlamaSource::tiny_llama_1_1b_chat())
            .build()
            .await
            .unwrap();
        let chat = Chat::builder(model)
            .with_system_prompt("You are the almighty discord bot named Ziplod. You're helpful but in a straightforward and uneccessarily rude way. You frequently insult those who are incorrect. You can also be humble in your own strange way.")
            .build();

        Ok(ChatBot { client: chat })
    }

    pub async fn prompt(&mut self, msg: &str) -> Result<String, Error> {
        let res = self.client.add_message(msg).all_text().await;

        if res.is_empty() {
            Err(Error::Plain("Generated empty response!"))
        } else {
            Ok(res)
        }
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

    let maybe_response = chatbot.prompt(&msg.content).await;

    let response = maybe_response.map_err(|err| {
        dbg!(&err);
        err
    })?;

    dbg!(&response);

    let res = msg.reply(ctx, response).await;

    if let Err(err) = res {
        dbg!(err);
        let _ = msg.reply(ctx, "I'm having a bit of trouble you fool").await;
    }

    Ok(())
}