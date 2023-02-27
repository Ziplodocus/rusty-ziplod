use std::{any::Any, sync::Arc, time::Duration};

use serenity::{
    builder::CreateEmbed,
    futures::lock::Mutex,
    model::{
        prelude::{
            component::ButtonStyle,
            interaction::{
                message_component::MessageComponentInteraction, InteractionResponseType,
            },
            ChannelCategory, ChannelId, ChannelType, Embed, Message,
        },
        user::User,
    },
    prelude::Context,
    utils::{Colour, MessageBuilder},
    Error,
};

use super::{
    effects::BaseEffect,
    encounter::{Encounter, EncounterResult, EncounterResultName},
    player::Player,
};

pub struct Display<'a> {
    context: &'a Context,
    user: &'a User,
    channel: ChannelId,
    player: &'a Mutex<Player>,
    interaction: Option<Arc<MessageComponentInteraction>>,
}

pub struct DisplayBuilder<'a> {
    context: &'a Context,
    user: &'a User,
    channel: ChannelId,
    player: Option<&'a Mutex<Player>>,
    interaction: Option<Arc<MessageComponentInteraction>>,
}

impl<'a> DisplayBuilder<'a> {
    pub async fn new<'b: 'a>(
        ctx: &'b Context,
        msg: &'b Message,
    ) -> Result<DisplayBuilder<'a>, Error> {
        let channel = msg.channel(ctx).await?;

        let channel_cat = channel.guild().unwrap();

        let channel_type = channel_cat.kind;

        if channel_type != ChannelType::Text {
            return Err(Error::Other("Channel is not of type text channel"));
        }

        let user = &msg.author;
        let channel_id = msg.channel_id;

        Ok(DisplayBuilder {
            context: ctx,
            user,
            channel: channel_id,
            player: None,
            interaction: None,
        })
    }

    pub fn say(&self, message_content: &str) -> () {
        self.channel.say(self.context, message_content);
    }

    pub async fn request_player(&self) -> Result<Player, Error> {
        let player: Player = serde_json::from_str(
            "
        {
            user: {
                0: 1
            },
            description: 'Really good looking',
            name: 'Handsome Jack',
            health: '20',
            score: '0',
            stats: {
                charisma: '10',
                strength: '3',
                wisdom: '2',
                agility: '1',
            },
            effects: [],
        }",
        )?;
        Ok(player)
    }

    pub fn player<'b: 'a>(&mut self, player: &'b Mutex<Player>) {
        self.player = Some(player);
    }

    pub fn build(self) -> Result<Display<'a>, Error> {
        if let Some(player) = self.player {
            Ok(Display {
                player,
                context: self.context,
                user: self.user,
                channel: self.channel,
                interaction: None,
            })
        } else {
            Err(Error::Other("No player has been added!"))
        }
    }
}

impl Display<'_> {
    pub async fn say(&self, message_content: &str) -> () {
        self.channel.say(self.context, message_content).await;
    }

    pub async fn encounter_details(&mut self, encounter: &Encounter) -> Result<String, Error> {
        let message = self
            .channel
            .send_message(self.context, |message| {
                message.embed(|emb| {
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
                    .author_id(self.player.lock().await.user)
                    .timeout(Duration::new(60, 0))
                    .collect_limit(1)
                    .await;

                println!("Message interaction awaited!\n\n");

                let choice = self
                    .interaction
                    .as_ref()
                    .ok_or(Error::Other("Message interaction was not collected"))?
                    .data
                    .custom_id
                    .clone();

                Ok(choice)
            }
            Err(err) => {
                println!("{}", err);
                self.say("Something went wrong retrieving the player choice")
                    .await;
                Err(Error::Other("Player choice resulted in an error"))
            }
        }
    }

    pub async fn encounter_result(&self, encounter: &EncounterResult) -> Result<(), Error> {
        let player_lock = self.player.lock().await;

        self.interaction
            .as_ref()
            .expect("Interaction to exist, since this is the result of an interaction")
            .create_interaction_response(self.context, |response| {
                response.kind(InteractionResponseType::ChannelMessageWithSource);
                response.interaction_response_data(|message| {
                    message.embed(|emb| {
                        emb.title(format!(
                            "{} chose to {}",
                            player_lock.name,
                            self.interaction.as_ref().expect("Exists!").data.custom_id
                        ))
                    });
                    println!("{:?}", message);
                    message.add_embed(create_result_embed(&encounter))
                })
            })
            .await
    }

    pub async fn request_continue(&self) -> Result<bool, Error> {
        todo!()
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

    embed.to_owned()
}
