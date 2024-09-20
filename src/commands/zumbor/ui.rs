use std::{collections::VecDeque, fmt, panic, sync::Arc, time::Duration};

use serenity::{
    all::{
        ChannelId, CreateActionRow, CreateButton, CreateMessage, EditMessage, Interaction, Message,
    },
    builder::CreateEmbed,
    prelude::Context,
};

use crate::{errors::Error, utilities::await_interactions};

use super::{
    encounter::{Encounter, EncounterResult},
    player::Player,
};

pub struct UI<'a> {
    context: &'a Context,
    channel: ChannelId,
    interaction: Option<Arc<Interaction>>,
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
    ) -> Result<(String, Message), crate::errors::Error> {
        println!("{:?}", player);
        let user_tag = player.tag.clone();

        let message = self
            .channel
            .send_message(
                self.context,
                CreateMessage::new()
                    .embeds(vec![player.into(), encounter.into()])
                    .components(vec![encounter.into()]),
            )
            .await;

        match message {
            Ok(message) => {
                self.interaction = Some(Arc::new(Interaction::Component(
                    await_interactions::component(self.context, &message, Arc::from(user_tag))
                        .await?,
                )));

                let choice = self
                    .interaction
                    .as_ref()
                    .expect("Interaction has just been set!")
                    .as_message_component()
                    .ok_or(Error::Plain("Message interaction was not collected"))?
                    .data
                    .custom_id
                    .clone();

                Ok((choice, message))
            }
            Err(err) => {
                println!("{}", err);
                self.say("Something went wrong sending encounter details")
                    .await;
                Err(Error::Plain("Player choice resulted in an error"))
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
            .edit(
                self.context,
                EditMessage::new()
                    .add_embeds(vec![
                        CreateEmbed::new().title(format!(
                            "{} chose to {}",
                            player.name,
                            self.interaction
                                .as_ref()
                                .expect("Exists!")
                                .as_message_component()
                                .expect("It can only be a message component")
                                .data
                                .custom_id
                        )),
                        result.into(),
                    ])
                    .add_embeds(self.get_queued_messages().into()),
            )
            .await?;

        Ok(message)
    }

    pub async fn request_continue(&self, player: &Player) -> Result<ContinueOption, Error> {
        println!("Requesting continue!");
        let message = self
            .channel
            .send_message(
                self.context,
                CreateMessage::new().components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(ContinueOption::Continue.to_string())
                        .label("Continue your journey"),
                    CreateButton::new(ContinueOption::Rest.to_string()).label("Take a break"),
                ])]),
            )
            .await;

        match message {
            Ok(message) => {
                let user_tag = player.tag.clone();
                let context = self.context.clone();

                let interaction = message
                    .await_component_interaction(self.context)
                    .filter(move |interaction| interaction.user.tag() == user_tag)
                    .timeout(Duration::new(120, 0))
                    .await
                    .ok_or(Error::Plain("Message interaction was not collected"))?;

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
                Err(Error::Plain("Player choice resulted in an error"))
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
            .send_message(self.context, CreateMessage::new().embeds(messages.into()))
            .await
            .map_err(|err| {
                dbg!(&err);
                Error::Serenity(err)
            })
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

impl From<ContinueOption> for &str {
    fn from(value: ContinueOption) -> Self {
        match value {
            ContinueOption::Continue => "continue",
            ContinueOption::Rest => "rest",
        }
    }
}
