use serenity::{
    all::{
        ActionRow, ActionRowComponent, ComponentInteraction, CreateActionRow, CreateButton,
        CreateEmbed, CreateInputText, CreateInteractionResponseMessage, CreateMessage, CreateModal,
        InputTextStyle, ModalInteraction,
    },
    builder::CreateInteractionResponse,
    model::prelude::{ChannelId, Message},
    prelude::Context,
};

use crate::{commands::zumbor::attributes::Attribute, errors::Error};

pub struct PlayerDetails {
    pub name: String,
    pub description: String,
}

impl TryFrom<&Vec<ActionRow>> for PlayerDetails {
    type Error = Error;
    fn try_from(details_data: &Vec<ActionRow>) -> Result<Self, Self::Error> {
        let mut name: Option<String> = None;
        let mut description: Option<String> = None;

        for row in details_data {
            let component = &row.components[0];
            let (key, value) = match component {
                ActionRowComponent::InputText(input) => {
                    (input.custom_id.clone(), input.value.clone())
                }
                _ => return Err(Error::Plain("Should be a text input")),
            };

            match key.as_str() {
                "name" => name = value,
                "description" => description = value,
                _ => {
                    return Err(Error::Plain(
                        "Key should be either \"name\" or \"description\"",
                    ))
                }
            };
        }

        Ok(PlayerDetails {
            name: name.clone().expect("Fields should be filled out"),
            description: description.clone().expect("Fields should be filled out"),
        })
    }
}

impl TryFrom<Vec<ActionRow>> for PlayerDetails {
    type Error = Error;

    fn try_from(details_data: Vec<ActionRow>) -> Result<Self, Self::Error> {
        let mut name: Option<String> = None;
        let mut description: Option<String> = None;

        for row in details_data {
            let component = &row.components[0];
            let (key, value) = match component {
                ActionRowComponent::InputText(input) => {
                    (input.custom_id.clone(), input.value.clone())
                }
                _ => return Err(Error::Plain("Should be a text input")),
            };

            match key.as_str() {
                "name" => name = value,
                "description" => description = value,
                _ => {
                    return Err(Error::Plain(
                        "Key should be either \"name\" or \"description\"",
                    ))
                }
            };
        }

        Ok(PlayerDetails {
            name: name.clone().expect("Fields should be filled out"),
            description: description.clone().expect("Fields should be filled out"),
        })
    }
}

pub async fn prompt_character_creation_start(
    channel: ChannelId,
    context: &Context,
) -> Result<Message, Error> {
    channel
        .send_message(
            context,
            CreateMessage::new()
                .add_embed(CreateEmbed::new().title("Create a character"))
                .button(CreateButton::new("choose_stats").label("Choose")),
        )
        .await
        .map_err(|err| {
            println!("{}", err);
            Error::Plain("Failed to send player request message")
        })
}

pub async fn prompt_for_player_stats(
    interaction: ModalInteraction,
    context: &Context,
) -> Result<(), Error> {
    interaction
        .create_response(
            context,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(CreateEmbed::new().title("Choose your stats..."))
                    .button(CreateButton::new("stats").label("Stats")),
            ),
        )
        .await
        .map_err(|e| {
            dbg!(e);
            Error::Plain("Modal failed")
        })
}

pub async fn re_prompt_for_player_stats(
    interaction: ModalInteraction,
    context: &Context,
) -> Result<(), Error> {
    interaction
        .create_response(
            context,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(
                        CreateEmbed::new()
                            .title("The stats you have chosen are too powerful for you..."),
                    )
                    .button(CreateButton::new("stats").label("Try Again")),
            ),
        )
        .await
        .map_err(|_e| Error::Plain("Modal failed"))
}

pub async fn prompt_with_character_details_modal(
    interaction: ComponentInteraction,
    context: &Context,
) -> Result<(), Error> {
    interaction
        .create_response(context, create_character_details_modal())
        .await
        .map_err(|err| {
            println!("{}", err);
            Error::Plain("Modal failed")
        })
}

pub async fn prompt_with_stats_modal(
    interaction: ComponentInteraction,
    context: &Context,
) -> Result<(), Error> {
    interaction
        .create_response(context, create_stats_modal())
        .await
        .map_err(|err| {
            println!("Stats modal fail... {}", err);
            Error::Plain("Modal failed")
        })
}

fn create_character_details_modal() -> CreateInteractionResponse {
    CreateInteractionResponse::Modal(
        CreateModal::new("character_details", "Who are you adventurer?").components(Vec::from([
            CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, "Name", "name")),
            CreateActionRow::InputText(CreateInputText::new(
                InputTextStyle::Paragraph,
                "Description",
                "description",
            )),
        ])),
    )
}

fn create_stats_modal() -> CreateInteractionResponse {
    CreateInteractionResponse::Modal(
        CreateModal::new("stats", "Allocate your 5 stat points").components(
            Attribute::VALUES
                .map(|attr| -> CreateActionRow {
                    let name: String = attr.into();
                    CreateActionRow::InputText(
                        CreateInputText::new(InputTextStyle::Short, &name, &name).required(true),
                    )
                })
                .to_vec(),
        ),
    )
}
