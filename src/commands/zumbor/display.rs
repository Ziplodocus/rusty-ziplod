use std::{
    any::Any, cmp, collections::VecDeque, fmt, ops::Deref, panic, str::FromStr, sync::Arc,
    time::Duration,
};

use serenity::{
    builder::{self, CreateComponents, CreateEmbed},
    futures::lock::Mutex,
    model::{
        prelude::{
            component::ButtonStyle,
            interaction::{
                message_component::MessageComponentInteraction, InteractionResponseType,
            },
            ChannelCategory, ChannelId, ChannelType, Embed, Message, UserId,
        },
        user::User,
    },
    prelude::Context,
    utils::{Colour, MessageBuilder},
    Error,
};

use super::{
    effects::{Attribute, BaseEffect},
    encounter::{Encounter, EncounterResult, EncounterResultName},
    player::Player,
};

pub struct Display<'a> {
    context: &'a Context,
    channel: ChannelId,
    player: &'a Mutex<Player>,
    interaction: Option<Arc<MessageComponentInteraction>>,
    messages: VecDeque<CreateEmbed>,
}

impl Display<'_> {
    pub fn builder() -> DisplayBuilder<'static> {
        DisplayBuilder::default()
    }

    pub async fn say(&self, message_content: &str) -> () {
        self.channel.say(self.context, message_content).await;
    }

    pub async fn encounter_details(
        &mut self,
        encounter: &Encounter,
    ) -> Result<(String, Message), Error> {
        let player = self.player.lock().await.clone();
        let user_tag = player.tag.clone();

        let message = self
            .channel
            .send_message(self.context, |message| {
                message.set_embed(player.into()).add_embed(|emb| {
                    emb.title(&encounter.title)
                        .description(&encounter.text)
                        .color(encounter.color.unwrap_or_default())
                });

                message.components(|components| {
                    components.create_action_row(|row| {
                        for (label, _) in &encounter.options {
                            row.create_button(|but| {
                                but.custom_id(&label)
                                    .label(&label)
                                    .style(ButtonStyle::Primary)
                            });
                        }
                        row
                    })
                })
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
        mut message: Message,
    ) -> Result<Message, Error> {
        let player = self.player.lock().await;
        let name = player.name.clone();

        message.embeds.remove(0);

        message
            .edit(self.context, |msg| {
                msg.add_embed(|emb| {
                    emb.title(format!(
                        "{} chose to {}",
                        name,
                        self.interaction.as_ref().expect("Exists!").data.custom_id
                    ))
                })
                .add_embed(|emb| {
                    if let Some(effect) = &result.base_effect {
                        match effect {
                            BaseEffect::Stat(eff) => {
                                emb.field(&eff.name, eff.potency, true);
                            }
                            BaseEffect::Health(eff) => {
                                emb.field("Health", eff.potency, true);
                            }
                        }
                    }

                    emb.title(&result.title)
                        .description(&result.text)
                        .colour(match &result.kind {
                            EncounterResultName::Success(_) => Colour::from((20, 240, 60)),
                            EncounterResultName::Fail(_) => Colour::from((240, 40, 20)),
                        })
                })
                .add_embeds(self.get_queued_messages().into_iter().collect())
                .components(|comp| comp)
            })
            .await?;
        Ok(message)
    }

    pub async fn request_continue(&self) -> Result<ContinueOption, Error> {
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
                let context = self.context.clone();

                let player_guard = self.player.lock().await;
                let user_tag: String = player_guard.deref().tag.clone();
                drop(player_guard);

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

    pub async fn send_player_info(&self) -> Result<Message, Error> {
        let player_guard = self.player.lock().await;
        let player: Player = player_guard.deref().clone();

        self.channel
            .send_message(self.context, |msg| {
                msg.set_embed(player.into());
                msg
            })
            .await
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
        self.channel.send_message(self.context, |message| {
            message.set_embeds(messages.into())
        }).await
    }
}

#[derive(Default)]
pub struct DisplayBuilder<'a> {
    context: Option<&'a Context>,
    channel: Option<ChannelId>,
    player: Option<&'a Mutex<Player>>,
}

impl<'a> DisplayBuilder<'a> {
    pub fn context(mut self, context: &'a Context) -> Self {
        self.context = Some(&context);
        self
    }

    pub fn channel(mut self, channel: ChannelId) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn player(mut self, player: &'a Mutex<Player>) -> Self {
        self.player = Some(player);
        self
    }

    pub fn build(self) -> Display<'a> {
        Display {
            player: self
                .player
                .expect("Player should be added to the builder before building"),
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

fn create_result_embed(result: &EncounterResult) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    embed
        .title(&result.title)
        .description(&result.text)
        .colour(match &result.kind {
            EncounterResultName::Success(_) => Colour::from((20, 240, 60)),
            EncounterResultName::Fail(_) => Colour::from((240, 40, 20)),
        });

    if let Some(effect) = &result.base_effect {
        match effect {
            BaseEffect::Stat(eff) => {
                embed.field(&eff.name, eff.potency, true);
            }
            BaseEffect::Health(eff) => {
                embed.field("Health", eff.potency, true);
            }
        }
    }

    embed
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
