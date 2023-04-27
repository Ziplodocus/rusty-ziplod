use core::panic;
use std::cmp;

use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::{
    builder::CreateEmbed,
    model::{prelude::{ChannelId, component::{ActionRow, ActionRowComponent}}},
    prelude::{Context},
    Error, FutureExt,
};

use crate::StorageClient;

use super::effects::{Attribute, Effectable, LingeringEffect, LingeringEffectName};

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub tag: String,
    pub description: String,
    pub name: String,
    pub health: i16,
    pub score: u16,
    pub stats: Stats,
    pub effects: Vec<LingeringEffect>,
}

impl Player {
    pub fn new(tag: String, details : PlayerDetails, stats : Stats) -> Player {
        let PlayerDetails {name, description} = details;
        Player {
            tag,
            health: 20,
            score: 0,
            effects: Vec::new(),
            stats,
            name,
            description
        }
    }

    pub fn add_score(&mut self, score: u16) {
        self.score += score
    }

    pub async fn remove(&self) -> Result<&str, Error> {
        todo!()
    }

    pub async fn save(&self) -> Result<bool, Error> {
        todo!()
    }

    pub fn roll_stat(&self, stat: &Attribute) -> RollResult {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(1..20);

        let critical = if (roll == 1) | (roll == 20) {
            true
        } else {
            false
        };

        let value = roll + self.stats.get(stat.clone());

        RollResult { critical, value }
    }
}

impl From<Player> for CreateEmbed {
    fn from(player: Player) -> Self {
        let mut embed = CreateEmbed::default();

        // Determining color of embed from players health
        let current_health: u8 = player.health.try_into().unwrap_or(255);

        let color: (u8, u8, u8) = (
            cmp::min(
                cmp::max(255u8 - (cmp::min(current_health, 20) / 20 * 255), 0),
                255,
            ),
            cmp::max(cmp::min((cmp::min(current_health, 20) / 20) * 255, 255), 0),
            0,
        );

        use Attribute::{Agility, Charisma, Strength, Wisdom};

        embed
            .author(|author| author.name(player.tag))
            .title(player.name)
            .description(player.description)
            .color(color)
            .field("Score", player.score, true)
            .field("Health", player.health, true)
            .field(Charisma, player.stats.get(Charisma), true)
            .field(Strength, player.stats.get(Strength), true)
            .field(Wisdom, player.stats.get(Wisdom), true)
            .field(Agility, player.stats.get(Agility), true);

        embed
    }
}

impl Effectable for Player {
    fn get_effects(&self) -> Vec<LingeringEffect> {
        self.effects.clone()
    }
    fn set_effects(&mut self, effects: Vec<LingeringEffect>) {
        self.effects = effects;
    }
    fn get_health(&self) -> i16 {
        self.health
    }
    fn get_stats(&self) -> Stats {
        self.stats.clone()
    }
    fn set_health(&mut self, health: i16) {
        self.health = health;
    }
    fn set_stats(&mut self, stats: Stats) {
        self.stats = stats;
    }
}
pub struct PlayerDetails {
    name: String,
    description: String
}

impl TryFrom<Vec<ActionRow>> for PlayerDetails {
    type Error = serenity::Error;
    fn try_from(details_data: Vec<ActionRow>) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut description = None;
        for row in details_data {
            let component = &row.components[0];
            let (key, value) = match component {
                ActionRowComponent::InputText(pair) => (pair.custom_id.clone(), pair.value.clone()),
                _ => return Err(Error::Other("Should be a text input")),
            };
            match key.try_into().unwrap() {
                InputIds::Name => name = Some(value),
                InputIds::Description => description = Some(value),
                _ => return Err(Error::Other("Should be a InputID enum variant Name or Description"))
            }
        };

        Ok(PlayerDetails {
            name: name.expect("Fields should be filled out"),
            description: description.expect("Fields should be filled out")
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub charisma: i16,
    pub strength: i16,
    pub wisdom: i16,
    pub agility: i16,
}

impl Stats {
    pub fn builder() -> StatsBuilder {
        StatsBuilder::default()
    }
    pub fn get(&self, key: Attribute) -> &i16 {
        match key {
            Attribute::Charisma => &self.charisma,
            Attribute::Strength => &self.strength,
            Attribute::Wisdom => &self.wisdom,
            Attribute::Agility => &self.agility,
        }
    }
    pub fn get_mut(&mut self, key: Attribute) -> &mut i16 {
        match key {
            Attribute::Charisma => &mut self.charisma,
            Attribute::Strength => &mut self.strength,
            Attribute::Wisdom => &mut self.wisdom,
            Attribute::Agility => &mut self.agility,
        }
    }
}

impl TryFrom<Vec<ActionRow>> for Stats {
    type Error = serenity::Error;
    fn try_from(stats_data: Vec<ActionRow>) -> Result<Self, Self::Error> {
        let mut create_stats = Stats::builder();
    for row in stats_data {
        let component = &row.components[0];
        let (name, value) = match component {
            ActionRowComponent::InputText(pair) => (pair.custom_id.clone(), pair.value.clone()),
            _ => panic!("Field is not an input text field"),
        };
        let value = value.parse::<i16>().map_err(|_e| Error::Other("Failed to parse string as i16"))?;

        create_stats.set(name.try_into().expect("Id of input to be an attribute"), value);
    };
    create_stats.build()
    }
}

#[derive(Default)]
pub struct StatsBuilder {
    charisma: Option<i16>,
    strength: Option<i16>,
    wisdom: Option<i16>,
    agility: Option<i16>
}
impl StatsBuilder {
    pub fn set(&mut self, key: Attribute, value: i16) -> &mut StatsBuilder {
        match key {
            Attribute::Charisma => self.charisma = Some(value),
            Attribute::Strength => self.strength = Some(value),
            Attribute::Wisdom => self.wisdom = Some(value),
            Attribute::Agility => self.agility = Some(value),
        };
        self
    }

    pub fn build(self) -> Result<Stats, Error> {
        Ok(Stats {
            charisma: self.charisma.ok_or_else(|| Error::Other("Oh no"))?,
            strength: self.strength.ok_or_else(|| Error::Other("Oh no"))?,
            wisdom: self.wisdom.ok_or_else(|| Error::Other("Oh no"))?,
            agility: self.agility.ok_or_else(|| Error::Other("Oh no"))?
        })
    }
}

pub enum PlayerEvent {
    EffectStart(LingeringEffectName),
    EffectEnd(LingeringEffectName),
    EffectApplied(LingeringEffect),
}

pub struct RollResult {
    pub critical: bool,
    pub value: i16,
}

// Gets the player's save if it exists
pub async fn fetch(ctx: &Context, user_tag: String) -> Result<Player, Error> {
    // return Err(Error::Other("Manually failing get to test request player"));
    let data = ctx.data.read().await;

    let storage_client = data.get::<StorageClient>().unwrap();
    let path = "zumbor/saves/".to_string() + &user_tag + ".json";

    let bytes = storage_client.download(path).await?;

    let maybe_player: Result<Player, Error> =
        serde_json::from_slice(&bytes).map_err(|err| Error::Json(err));

    // V2 players should be serializable straight to a struct
    if let Ok(player) = maybe_player {
        return Ok(player);
    }

    println!("Failed deserialise of object as struct");

    // Handles first versions of the Player object
    let maybe_player_map: Value = serde_json::from_slice(&bytes).unwrap();

    let name: String = maybe_player_map
        .get("name")
        .ok_or(Error::Other("name field not present in data"))?
        .as_str()
        .expect("Name is a string")
        .to_string();
    let tag: String = maybe_player_map
        .get("user")
        .ok_or(Error::Other("name field not present in data"))?
        .as_str()
        .expect("User is a string")
        .to_string();
    let description: String = maybe_player_map
        .get("description")
        .ok_or(Error::Other("description field not present in data"))?
        .as_str()
        .expect("Description is a string")
        .to_string();
    let health: i16 = maybe_player_map
        .get("health")
        .ok_or(Error::Other("health field not present in data"))?
        .as_u64()
        .expect("Health is a number")
        .try_into()
        .unwrap();
    let score: u16 = maybe_player_map
        .get("score")
        .ok_or(Error::Other("score field not present in data"))?
        .as_u64()
        .expect("Score to be a number")
        .try_into()
        .unwrap();

    let stats = if let Value::Object(maybe_stats) = &maybe_player_map["stats"] {
        Ok(Stats {
            charisma: maybe_stats["Charisma"]
                .as_u64()
                .expect("Stat to be a number")
                .try_into()
                .expect("Good number"),
            strength: maybe_stats["Strength"]
                .as_u64()
                .expect("Stat to be a number")
                .try_into()
                .expect("Good number"),
            wisdom: maybe_stats["Wisdom"]
                .as_u64()
                .expect("Stat to be a number")
                .try_into()
                .expect("Good number"),
            agility: maybe_stats["Agility"]
                .as_u64()
                .expect("Stat to be a number")
                .try_into()
                .expect("Good number"),
        })
    } else {
        Err(Error::Other(("Stats should be an object / hash map")))
    }?;

    println!("Starting desrialise of effects..");
    let effects: Vec<LingeringEffect> =
        serde_json::from_value(maybe_player_map["effects"].clone()).unwrap_or(Vec::new());

    let player = Player {
        tag,
        name,
        description,
        health: health.try_into().unwrap(),
        score,
        stats,
        effects,
    };

    Ok(player)
}

pub async fn save(ctx: &Context, player: &Player) -> Result<(), Error> {
    let data = ctx.data.read().await;

    let storage_client = data
        .get::<StorageClient>()
        .ok_or(Error::Other("Storage client not accessible!"))?;

    let player_json: String = serde_json::to_string(player).map_err(|err| Error::from(err))?;
    let save_name = "zumbor/saves/".to_string() + &player.tag;

    storage_client.upload_json(save_name, player_json).await
}

pub async fn request_player(
    channel: ChannelId,
    user_tag: String,
    context: &Context,
) -> Result<Player, Error> {
    let message = messages::character_details_request(channel, context).await?;

    let interaction = await_interation::component(&message, context, user_tag.clone()).await?;

    send_modal::character_details(interaction, context).await?;

    let interaction = await_interation::modal(&message, context, user_tag.clone()).await?;

    let details_data = interaction.data.components.clone();

    messages::stats_request(interaction, context).await?;

    let interaction = await_interation::component(&message, context, user_tag.clone()).await?;

    send_modal::stats(interaction, context).await?;

    let interaction = await_interation::modal(&message, context, user_tag.clone()).await?;

    message.delete(context).await;

    let stats_data = interaction.data.components.clone();

    let stats : Stats = stats_data.try_into()?;

    let details : PlayerDetails = details_data.try_into()?;

    Ok(Player::new(user_tag, details, stats))
}

mod send_modal {
    use std::sync::Arc;

    use serenity::{
        builder::CreateInteractionResponse,
        model::prelude::{
            component::InputTextStyle,
            interaction::{
                message_component::MessageComponentInteraction, InteractionResponseType,
            },
        },
        prelude::Context,
        Error,
    };

    use crate::commands::zumbor::effects::Attribute;

    use super::InputIds;

    pub async fn stats(
        interaction: Arc<MessageComponentInteraction>,
        context: &Context,
    ) -> Result<(), Error> {
        interaction
            .create_interaction_response(context, create_stats_modal)
            .await
            .map_err(|err| {
                println!("{}", err);
                Error::Other("Modal failed")
            })
    }

    fn create_stats_modal<'a, 'b>(
        response: &'a mut CreateInteractionResponse<'b>,
    ) -> &'a mut CreateInteractionResponse<'b> {
        response
            .kind(InteractionResponseType::Modal)
            .interaction_response_data(|data| {
                data.title("Allocate your 5 stat points").custom_id("stats")
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

    pub async fn character_details(
        interaction: Arc<MessageComponentInteraction>,
        context: &Context,
    ) -> Result<(), Error> {
        interaction
            .create_interaction_response(context, create_character_details_modal)
            .await
            .map_err(|err| {
                println!("{}", err);
                Error::Other("Modal failed")
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
                                    .custom_id(InputIds::Name)
                                    .required(true)
                                    .style(InputTextStyle::Short)
                            })
                        })
                        .create_action_row(|row| {
                            row.create_input_text(|inp| {
                                inp.label("Description")
                                    .custom_id(InputIds::Description)
                                    .required(true)
                                    .style(InputTextStyle::Paragraph)
                            })
                        })
                    })
            })
    }
}

mod await_interation {
    use std::{sync::Arc, time::Duration};

    use serenity::{
        model::prelude::{
            interaction::{
                message_component::MessageComponentInteraction, modal::ModalSubmitInteraction,
            },
            Message,
        },
        prelude::Context,
        Error,
    };

    pub(crate) async fn component(
        message: &Message,
        context: &Context,
        user_tag: String,
    ) -> Result<Arc<MessageComponentInteraction>, Error> {
        message
            .await_component_interaction(context)
            .filter(move |interaction| interaction.user.tag() == user_tag)
            .timeout(Duration::new(120, 0))
            .collect_limit(1)
            .await
            .ok_or(Error::Other(
                "Message Component interaction was not collected",
            ))
    }
    pub(crate) async fn modal(
        message: &Message,
        context: &Context,
        user_tag: String,
    ) -> Result<Arc<ModalSubmitInteraction>, Error> {
        message
            .await_modal_interaction(context)
            .filter(move |interaction| interaction.user.tag() == user_tag)
            .timeout(Duration::new(120, 0))
            .collect_limit(1)
            .await
            .ok_or(Error::Other("Modal interaction was not collected"))
    }
}

mod messages {
    use std::sync::Arc;

    use serenity::{
        model::prelude::{
            component::ButtonStyle,
            interaction::{modal::ModalSubmitInteraction, InteractionResponseType},
            ChannelId, Message,
        },
        prelude::Context,
        Error,
    };

    pub(crate) async fn character_details_request(
        channel: ChannelId,
        context: &Context,
    ) -> Result<Message, Error> {
        channel
            .send_message(context, |msg| {
                msg.add_embed(|embed| embed.title("Create a Character"))
                    .components(|components| {
                        components.create_action_row(|row| {
                            row.create_button(|button| {
                                button.custom_id("choose_stats").label("Choose")
                            })
                        })
                    })
            })
            .await
            .map_err(|err| {
                println!("{}", err);
                Error::Other("Failed to send player request message")
            })
    }

    pub(crate) async fn stats_request(
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
            .map_err(|_e| Error::Other("Modal failed"))
    }
}

enum InputIds {
    Name,
    Description,
}

impl ToString for InputIds {
    fn to_string(&self) -> String {
        match self {
            InputIds::Name => "name".to_owned(),
            InputIds::Description => "description".to_owned(),
        }
    }
}

impl TryFrom<String> for InputIds {
    type Error = String;
    fn try_from(value: String) -> Result<Self, String> {
        match value.as_str() {
            "name" => Ok(InputIds::Name),
            "description" => Ok(InputIds::Description),
            _ => Err("Doesn't translate to an Input ID".to_owned())
        }
    }
}