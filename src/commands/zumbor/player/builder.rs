use serenity::{
    builder::CreateInteractionResponse,
    model::{
        application::interaction::message_component::MessageComponentInteraction,
        prelude::{
            component::{ActionRow, ActionRowComponent, ButtonStyle, InputTextStyle},
            interaction::{modal::ModalSubmitInteraction, InteractionResponseType},
            ChannelId, Message,
        },
    },
    prelude::Context,
};
use std::sync::Arc;

use crate::{commands::zumbor::attributes::Attribute, errors::Error};

pub struct PlayerDetails {
    pub name: String,
    pub description: String,
}

impl TryFrom<&Vec<ActionRow>> for PlayerDetails {
    type Error = Error;
    fn try_from(details_data: &Vec<ActionRow>) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut description = None;
        for row in details_data {
            let component = &row.components[0];
            let (key, value) = match component {
                ActionRowComponent::InputText(pair) => (&pair.custom_id, &pair.value),
                _ => return Err(Error::Plain("Should be a text input")),
            };

            match key.as_str() {
                "name" => name = Some(value),
                "description" => description = Some(value),
                _ => {
                    return Err(Error::Plain(
                        "Key should be either \"name\" or \"description\"",
                    ))
                }
            };
        }

        Ok(PlayerDetails {
            name: name.expect("Fields should be filled out").as_str().into(),
            description: description
                .expect("Fields should be filled out")
                .as_str()
                .into(),
        })
    }
}

pub async fn prompt_character_creation_start(
    channel: ChannelId,
    context: &Context,
) -> Result<Message, Error> {
    channel
        .send_message(context, |msg| {
            msg.add_embed(|embed| embed.title("Create a Character"))
                .components(|components| {
                    components.create_action_row(|row| {
                        row.create_button(|button| button.custom_id("choose_stats").label("Choose"))
                    })
                })
        })
        .await
        .map_err(|err| {
            println!("{}", err);
            Error::Plain("Failed to send player request message")
        })
}

pub async fn prompt_for_player_stats(
    interaction: Arc<ModalSubmitInteraction>,
    context: &Context,
) -> Result<(), Error> {
    interaction
        .create_interaction_response(context, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| {
                    message
                        .embed(|embed| embed.title("Choose your stats..."))
                        .components(|comps| {
                            comps.create_action_row(|row| {
                                row.create_button(|button| {
                                    button
                                        .custom_id("stats")
                                        .label("Stats")
                                        .style(ButtonStyle::Primary)
                                })
                            })
                        })
                })
        })
        .await
        .map_err(|_e| {
            dbg!(_e);
            Error::Plain("Modal failed")
        })
}

pub async fn re_prompt_for_player_stats(
    interaction: Arc<ModalSubmitInteraction>,
    context: &Context,
) -> Result<(), Error> {
    interaction
        .create_interaction_response(context, |response| {
            response
                .kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| {
                    message
                        .embed(|embed| {
                            embed.title("The stats you have chosen are too powerful for you...")
                        })
                        .components(|comps| {
                            comps.create_action_row(|row| {
                                row.create_button(|button| {
                                    button
                                        .custom_id("stats")
                                        .label("Choose Again")
                                        .style(ButtonStyle::Primary)
                                })
                            })
                        })
                })
        })
        .await
        .map_err(|_e| Error::Plain("Modal failed"))
}
pub async fn prompt_with_character_details_modal(
    interaction: Arc<MessageComponentInteraction>,
    context: &Context,
) -> Result<(), Error> {
    interaction
        .create_interaction_response(context, create_character_details_modal)
        .await
        .map_err(|err| {
            println!("{}", err);
            Error::Plain("Modal failed")
        })
}

fn create_character_details_modal<'a, 'b>(
    response: &'a mut CreateInteractionResponse<'b>,
) -> &'a mut CreateInteractionResponse<'b> {
    response
        .kind(InteractionResponseType::Modal)
        .interaction_response_data(|data| {
            data.title("Who are you adventurer?")
                .custom_id("character_details")
                .components(|comp| {
                    comp.create_action_row(|row| {
                        row.create_input_text(|inp| {
                            inp.label("Name")
                                .custom_id("name")
                                .required(true)
                                .style(InputTextStyle::Short)
                        })
                    })
                    .create_action_row(|row| {
                        row.create_input_text(|inp| {
                            inp.label("Description")
                                .custom_id("description")
                                .required(true)
                                .style(InputTextStyle::Paragraph)
                        })
                    })
                })
        })
}

pub async fn prompt_with_stats_modal(
    interaction: Arc<MessageComponentInteraction>,
    context: &Context,
) -> Result<(), Error> {
    interaction
        .create_interaction_response(context, create_stats_modal)
        .await
        .map_err(|err| {
            println!("Stats modal fail... {}", err);
            Error::Plain("Modal failed")
        })
}

fn create_stats_modal<'a, 'b>(
    response: &'a mut CreateInteractionResponse<'b>,
) -> &'a mut CreateInteractionResponse<'b> {
    response
        .kind(InteractionResponseType::Modal)
        .interaction_response_data(|data| {
            data.title("Allocate your 5 stat points")
                .content("The total of your stats must be less than 5, and no individual stat may exceed 5")
                .custom_id("stats")
                .components(|comp| {
                    Attribute::VALUES.into_iter().for_each(|attr| {
                        comp.create_action_row(|row| {
                            let stat: String = attr.into();
                            row.create_input_text(|inp| {
                                inp.custom_id(&stat)
                                    .label(&stat)
                                    .style(InputTextStyle::Short)
                                    .required(true)
                            })
                        });
                    });
                    comp
                })
        })
}
