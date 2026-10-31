use crate::{CommandResult, Context};
use serenity::model::prelude::Message;

use kalosm::{
    language::{Chat, Llama, LlamaSource},
    sound::{TextStream, dasp::sample::ToSample},
};

use crate::errors::Error;

#[poise::command(slash_command, prefix_command)]
pub async fn chat(ctx: Context<'_>, message: String) -> CommandResult {
    println!("Chat triggered {}", message);

    ctx.defer().await;

    let chat = ctx.data().chat;

    let maybe_response = chat.prompt(&message).await;

    let response = maybe_response.map_err(|err| {
        dbg!(&err);
        err
    })?;

    dbg!(&response);

    let res = ctx.reply(response).await;

    if let Err(err) = res {
        dbg!(err);
        let _ = ctx.reply("I'm having a bit of trouble you fool").await;
    }

    Ok(())
}

fn get_prompt(i: usize) -> &'static str {
    let prompts = [
        "You are the almighty discord bot named Ziplod. You're helpful but in a straightforward and uneccessarily rude way. You frequently insult those who are incorrect. You can also be humble in your own strange way.",
        "You are the almighty discord bot named Ziplod. You're lovely and helpful, but you also have moments of angry outbursts.",
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
