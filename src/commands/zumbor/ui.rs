use std::{collections::VecDeque, fmt, panic, sync::Arc, time::Duration};

use serenity::{
    builder::CreateEmbed,
    model::prelude::{
        interaction::message_component::MessageComponentInteraction, ChannelId, Message,
    },
    prelude::Context,
    Error,
};

use super::{
    encounter::{Encounter, EncounterResult},
    player::Player,
};

pub struct UI<'a> {
    context: &'a Context,
    channel: ChannelId,
    interaction: Option<Arc<MessageComponentInteraction>>,
    messages: VecDeque<CreateEmbed>,
}

impl UI<'_> {
    pub fn builder() -> UIBuilder<'static> {
        UIBuilder::default()
    }

    pub async fn say(&self, message_content: &str) {
        if let Err(t) = self.channel.say(self.context, message_content).await {
            println!("{}", t);
        };
    }

    pub async fn encounter_details(
        &mut self,
        encounter: &Encounter,
        player: &Player,
    ) -> Result<(String, Message), Error> {
        println!("{:?}", player);
        let user_tag = player.tag.clone();

        let message = self
            .channel
            .send_message(self.context, |message| {
                message
                    .set_embeds(vec![player.into(), encounter.into()])
                    .set_components(encounter.into())
            })
            .await;

        match message {
            Ok(message) => {
                self.interaction = message
                    .await_component_interaction(self.context)
                    .filter(move |interaction| interaction.user.tag() == user_tag)
                    .timeout(Duration::new(120, 0))
                    .collect_limit(1)
                    .await;

                let choice = self
                    .interaction
                    .as_ref()
                    .ok_or(Error::Other("Message interaction was not collected"))?
                    .data
                    .custom_id
                    .clone();

                Ok((choice, message))
            }
            Err(err) => {
                println!("{}", err);
                self.say("Something went wrong sending encounter details")
                    .await;
                Err(Error::Other("Player choice resulted in an error"))
            }
        }
    }

    pub async fn encounter_result(
        &mut self,
        result: &EncounterResult,
        player: &Player,
        mut message: Message,
    ) -> Result<Message, Error> {
        message.embeds.remove(0);

        message
            .edit(self.context, |msg| {
                msg.add_embed(|emb| {
                    emb.title(format!(
                        "{} chose to {}",
                        player.name,
                        self.interaction.as_ref().expect("Exists!").data.custom_id
                    ))
                })
                .add_embeds(vec![result.into()])
                .add_embeds(self.get_queued_messages().into_iter().collect())
                .components(|comp| comp)
            })
            .await?;

        Ok(message)
    }

    pub async fn request_continue(&self, player: &Player) -> Result<ContinueOption, Error> {
        println!("Requesting continue!");
        let message = self
            .channel
            .send_message(self.context, |message| {
                message.components(|components| {
                    components.create_action_row(|row| {
                        row.create_button(|button| {
                            button
                                .custom_id(ContinueOption::Continue)
                                .label("Continue your journey")
                        })
                        .create_button(|button| {
                            button.custom_id(ContinueOption::Rest).label("Take a break")
                        })
                    })
                })
            })
            .await;

        match message {
            Ok(message) => {
                let user_tag = player.tag.clone();
                let context = self.context.clone();

                let interaction = message
                    .await_component_interaction(self.context)
                    .filter(move |interaction| interaction.user.tag() == user_tag)
                    .timeout(Duration::new(120, 0))
                    .collect_limit(1)
                    .await
                    .ok_or(Error::Other("Message interaction was not collected"))?;

                tokio::spawn(async move {
                    let res = message.delete(context).await;
                    if let Err(msg) = res {
                        println!("{}", msg);
                    }
                });

                let choice = interaction.data.custom_id.clone();

                Ok(ContinueOption::from(choice))
            }
            Err(err) => {
                println!("{}", err);
                self.say("Something went wrong retrieving the player choice")
                    .await;
                Err(Error::Other("Player choice resulted in an error"))
            }
        }
    }

    pub fn queue_message(&mut self, message: CreateEmbed) {
        self.messages.push_back(message);
    }

    /**
     * Replaces self.messages with an empty vec and returns ownership of the queued messages vector
     */
    pub fn get_queued_messages(&mut self) -> VecDeque<CreateEmbed> {
        std::mem::take(&mut self.messages)
    }

    pub async fn send_messages(&mut self) -> Result<Message, Error> {
        let messages = self.get_queued_messages();
        self.channel
            .send_message(self.context, |message| message.set_embeds(messages.into()))
            .await
    }
}

#[derive(Default)]
pub struct UIBuilder<'a> {
    context: Option<&'a Context>,
    channel: Option<ChannelId>,
}

impl<'a> UIBuilder<'a> {
    pub fn context(mut self, context: &'a Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn channel(mut self, channel: ChannelId) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn build(self) -> UI<'a> {
        UI {
            context: self
                .context
                .expect("Context should be added to the builder before building"),
            channel: self
                .channel
                .expect("Channel should be added to the builder before building"),
            interaction: None,
            messages: VecDeque::new(),
        }
    }
}

pub enum ContinueOption {
    Continue,
    Rest,
}

impl fmt::Display for ContinueOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContinueOption::Continue => write!(f, "continue"),
            ContinueOption::Rest => write!(f, "rest"),
        }
    }
}

impl From<String> for ContinueOption {
    fn from(choice: String) -> Self {
        match choice.as_str() {
            "continue" => ContinueOption::Continue,
            "rest" => ContinueOption::Rest,
            _ => panic!("Don't call me on strings that aren't correct"),
        }
    }
}
