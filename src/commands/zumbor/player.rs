use rand::Rng;
use serde::{Deserialize, Serialize};
use serenity::{builder::CreateEmbed, model::prelude::ChannelId, prelude::Context};
use std::{cmp, sync::Arc};

mod builder;
pub mod stats;
pub mod storage;
use super::{
    attributes::Attribute,
    effects::{Effectable, LingeringEffect},
};
use crate::{errors::Error, utilities::await_interactions};
use builder::PlayerDetails;
use stats::Stats;

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub fn new(tag: String, details: PlayerDetails, stats: Stats) -> Player {
        let PlayerDetails { name, description } = details;
        Player {
            tag,
            health: 20,
            score: 0,
            effects: Vec::new(),
            stats,
            name,
            description,
        }
    }

    pub fn add_score(&mut self, score: u16) {
        self.score += score
    }

    pub fn roll_stat(&self, stat: &Attribute) -> RollResult {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(1..20);

        match roll {
            1 => RollResult::CriticalFail,
            20 => RollResult::CriticalSuccess,
            num => RollResult::Value(num + self.stats.get(stat.clone())),
        }
    }
}

impl From<&Player> for CreateEmbed {
    fn from(player: &Player) -> Self {
        let mut embed = CreateEmbed::default();

        // Determining color of embed from players health
        let current_health: u8 = player.health.try_into().unwrap_or(255);

        let color: (u8, u8, u8) = (
            (255u8 - (cmp::min(current_health, 20) / 20 * 255)).clamp(0, 255),
            ((cmp::min(current_health, 20) / 20) * 255).clamp(0, 255),
            0,
        );

        use Attribute::{Agility, Charisma, Strength, Wisdom};

        embed
            .author(|author| author.name(&player.tag))
            .title(&player.name)
            .description(&player.description)
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

pub enum RollResult {
    CriticalFail,
    CriticalSuccess,
    Value(i16),
}

pub async fn create(
    context: &Context,
    user_tag: Arc<str>,
    channel: ChannelId,
) -> Result<Player, Error> {
    let message = builder::prompt_character_creation_start(channel, context).await?;
    let interaction = await_interactions::component(&message, context, user_tag.clone()).await?;
    builder::prompt_with_character_details_modal(interaction, context).await?;
    let interaction = await_interactions::modal(&message, context, user_tag.clone()).await?;
    let details_data = &interaction.data.components;

    builder::prompt_for_player_stats(interaction.clone(), context).await?;
    let interaction = await_interactions::component(&message, context, user_tag.clone()).await?;
    builder::prompt_with_stats_modal(interaction, context).await?;
    let interaction = await_interactions::modal(&message, context, user_tag.clone()).await?;
    let stats_data = interaction.data.components.clone();

    let mut stats: Stats = stats_data.try_into()?;

    println!("Sum is {} and max is {}", stats.sum(), stats.get_max());

    let mut loop_int = interaction.clone();
    println!("Is invalid?: {}", stats.sum() > 5 || stats.get_max() > 5);
    while stats.sum() > 5 || stats.get_max() > 5 {
        println!("Start loop");

        builder::re_prompt_for_player_stats(loop_int, context).await?;
        println!("Re request stats...");

        let interaction =
            await_interactions::component(&message, context, user_tag.clone()).await?;
        println!("Awaited button click...");

        builder::prompt_with_stats_modal(interaction, context).await?;
        println!("Sent next modal");

        loop_int = await_interactions::modal(&message, context, user_tag.clone()).await?;
        println!("Awaited modal interaction");

        let stats_data = loop_int.data.components.clone();

        stats = match stats_data.try_into() {
            Ok(val) => val,
            Err(_err) => continue,
        };

        println!("Sum is {} and max is {}", stats.sum(), stats.get_max());
    }

    if let Err(er) = message.delete(context).await {
        println!("Failed to delete previous message!: {}", er);
    };

    let details: PlayerDetails = details_data.try_into()?;

    Ok(Player::new(user_tag.to_string(), details, stats))
}

// pub enum PlayerEvent {
//     EffectStart(LingeringEffectName),
//     EffectEnd(LingeringEffectName),
//     EffectApplied(LingeringEffect),
// }
