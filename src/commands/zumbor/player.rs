use std::{cmp, collections::HashMap, thread::current};

use google_cloud_storage::http::objects::{
    download::Range, get::GetObjectRequest, list::ListObjectsRequest, Object,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::{
    builder::CreateEmbed,
    model::{prelude::UserId, user::User},
    prelude::Context,
    Error,
};

use derive_builder::Builder;

use crate::StorageClient;

use super::effects::{Attribute, Effectable, LingeringEffect, LingeringEffectName};

#[derive(Serialize, Deserialize, Builder, Clone)]
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
    pub fn add_score(&mut self, score: u16) {
        self.score += score
    }

    pub fn load(user: &User) -> Result<Self, Error> {
        // todo!()
        Ok(Player {
            tag: "BeefCake#2185".to_string(),
            description: "Really good looking".to_string(),
            name: "Handsome Jack".to_string(),
            health: 20,
            score: 0,
            stats: Stats {
                charisma: 10,
                strength: 3,
                wisdom: 2,
                agility: 1,
            },
            effects: Vec::new(),
        })
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

        // Determining color of emebed from players health
        let current_health: u8 = player.health.try_into().unwrap_or(255);
        dbg!(current_health);
        let color: (u8, u8, u8) = (
            cmp::min(cmp::max(255u8 - (current_health / 20 * 255), 0), 255),
            cmp::max(cmp::min((current_health / 20) * 255, 255), 0),
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Stats {
    pub charisma: i16,
    pub strength: i16,
    pub wisdom: i16,
    pub agility: i16,
}

impl Stats {
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

pub enum PlayerEvent {
    EffectStart(LingeringEffectName),
    EffectEnd(LingeringEffectName),
    EffectApplied(LingeringEffect),
}

pub struct RollResult {
    pub critical: bool,
    pub value: i16,
}

// Return a random encounter from the storage bucket
pub async fn get(ctx: &Context, user_tag: String) -> Result<Player, Error> {
    let data = ctx.data.read().await;

    let storage_client = data.get::<StorageClient>().unwrap();

    let client = &storage_client.client;

    let request = GetObjectRequest {
        bucket: "ziplod-assets".into(),
        object: "zumbor/saves/".to_string() + &user_tag + ".json", //&user_tag,
        ..Default::default()
    };

    let range = Range::default();

    let byte_array = client
        .download_object(&request, &range, None)
        .await
        .map_err(|_| Error::Other("User does not have a player saved"))?;

    let player: Result<Player, _> = serde_json::from_slice(&byte_array);

    // V2 players should be serializable straight to a struct
    if let Ok(player) = player {
        return Ok(player);
    }

    println!("Failed deserialise of object as struct");

    // Handles previous versions of the Encounter object
    let maybe_player_map: Value = serde_json::from_slice(&byte_array).unwrap();

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
