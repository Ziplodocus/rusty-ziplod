use rand::Rng;
use serde::{Deserialize, Serialize};
use serenity::{model::prelude::UserId, Error};

use derive_builder::Builder;

use super::effects::{Attribute, Effectable, LingeringEffect, LingeringEffectName};

#[derive(Serialize, Deserialize, Builder)]
pub struct Player {
    pub user: UserId,
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

    pub async fn load(user_id: UserId) -> Result<Self, Error> {
        // todo!()
        Ok(Player {
            user: user_id,
            description: "Really good looking".into(),
            name: "Handsome Jack".into(),
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
    charisma: i16,
    strength: i16,
    wisdom: i16,
    agility: i16,
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
