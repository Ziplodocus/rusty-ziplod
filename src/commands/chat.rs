use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::{Context, TypeMapKey},
};

use kalosm::{
    language::{Chat, Llama, LlamaSource},
    sound::{dasp::sample::ToSample, TextStream},
};

use crate::errors::Error;

fn get_prompt(i: usize) -> &'static str {
    let prompts = [
        "You are the almighty discord bot named Ziplod. You're helpful but in a straightforward and uneccessarily rude way. You frequently insult those who are incorrect. You can also be humble in your own strange way.",
        "You are the almighty discord bot named Ziplod. You're lovely and helpful, but you also have moments of angry outbursts."
    ];

    prompts[i]
}

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
            .with_system_prompt(get_prompt(0))
            .build();

        Ok(ChatBot { client: chat })
    }

    pub async fn prompt(&mut self, msg: &str) -> Result<String, Error> {
        let res = self.client.add_message(msg).all_text().await;

        if res.is_empty() {
            self.reset().await;
            Ok("I lost my memory. Please try again".to_string())
        } else {
            Ok(res)
        }
    }

    pub async fn reset(&mut self) {
        let model = Llama::builder()
            .with_source(LlamaSource::llama_7b_chat())
            .build()
            .await
            .unwrap();
        let chat = Chat::builder(model)
            .with_system_prompt(get_prompt(0))
            .build();

        self.client = chat;
    }
}

impl TypeMapKey for ChatBot {
    type Value = ChatBot;
}

#[command]
pub async fn chat(ctx: &Context, msg: &Message) -> CommandResult {
    let message = msg.content.replace("!chat", "");
    println!("Chat triggered {}", message);
    let mut data = ctx.data.write().await;
    let chatbot = data.get_mut::<ChatBot>().take().unwrap();

    let maybe_response = chatbot.prompt(&message).await;

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
