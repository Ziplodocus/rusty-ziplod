use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};
use serenity::{builder::CreateEmbed, utils::Colour};

use super::attributes::Attribute;

use super::player::stats::{Stats};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BaseEffect {
    Stat(BaseStatEffect),
    Health(BaseHealthEffect),
}

impl BaseEffect {
    pub fn get_potency(&self) -> i16 {
        match self {
            BaseEffect::Stat(eff) => eff.potency,
            BaseEffect::Health(eff) => eff.potency,
        }
    }
    pub fn set_potency(&mut self, potency: i16) {
        match self {
            BaseEffect::Stat(eff) => eff.potency = potency,
            BaseEffect::Health(eff) => eff.potency = potency,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BaseHealthEffect {
    pub potency: i16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BaseStatEffect {
    pub name: Attribute,
    pub potency: i16,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct LingeringEffect {
    pub kind: LingeringEffectType,
    pub name: LingeringEffectName,
    pub potency: i16,
    pub duration: i16,
}

impl From<&LingeringEffect> for CreateEmbed {
    fn from(effect: &LingeringEffect) -> Self {
        let LingeringEffect {
            name,
            kind,
            potency,
            duration,
        } = effect;

        let mut embed = CreateEmbed::default();

        embed
            .title(format!("{} {}", name, kind))
            .colour::<Colour>(name.into())
            .field("Potency", potency, true)
            .field("Duration", duration, true);

        embed
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum LingeringEffectName {
    Stat(Attribute),
    Poison,
    Regenerate,
}

impl Display for LingeringEffectName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Stat(attr) => write!(f, "{}", attr),
            Self::Poison => write!(f, "Poison"),
            Self::Regenerate => write!(f, "Regnerate"),
        }
    }
}

impl From<&LingeringEffectName> for Colour {
    fn from(effect_name: &LingeringEffectName) -> Self {
        match effect_name {
            LingeringEffectName::Poison => Colour::PURPLE,
            LingeringEffectName::Regenerate => Colour::FABLED_PINK,
            LingeringEffectName::Stat(attr) => match attr {
                Attribute::Strength => Colour::DARK_RED,
                Attribute::Wisdom => Colour::DARK_GREEN,
                Attribute::Agility => Colour::BLITZ_BLUE,
                Attribute::Charisma => Colour::DARK_GOLD,
            },
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum LingeringEffectType {
    Buff,
    Debuff,
}

impl Display for LingeringEffectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Buff => write!(f, "buff"),
            Self::Debuff => write!(f, "debuff"),
        }
    }
}

impl TryFrom<&str> for LingeringEffectType {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Buff" | "buff" => Ok(LingeringEffectType::Buff),
            "Debuff" | "debuff" => Ok(LingeringEffectType::Debuff),
            _ => Err("String is not related to a lingering effect type".into()),
        }
    }
}

pub trait Effectable {
    fn get_effects(&self) -> Vec<LingeringEffect>;
    fn set_effects(&mut self, effects: Vec<LingeringEffect>);
    fn get_health(&self) -> i16;
    fn set_health(&mut self, health: i16);
    fn get_stats(&self) -> Stats;
    fn set_stats(&mut self, stats: Stats);

    fn affect(&mut self, effect: &BaseEffect) {
        println!("Player affected by {:?}", effect);
        match effect {
            BaseEffect::Health(eff) => self.affect_health(eff),
            BaseEffect::Stat(eff) => self.affect_stat(eff),
        }
    }

    fn affect_health(&mut self, effect: &BaseHealthEffect) {
        println!(
            "Current Health {} changed by {}",
            self.get_health(),
            effect.potency
        );

        self.set_health(self.get_health() + effect.potency);

        println!("After Health: {}", self.get_health());
    }

    fn affect_stat(&mut self, effect: &BaseStatEffect) {
        let mut stats = self.get_stats();
        let stat = stats.get_mut(effect.name.clone());
        *stat += effect.potency;
        self.set_stats(stats);
    }

    fn add_effect(&mut self, effect: LingeringEffect) {
        let mut effects = self.get_effects();

        if let LingeringEffectName::Stat(attr) = &effect.name {
            self.affect_stat(&BaseStatEffect {
                name: attr.clone(),
                potency: effect.potency,
            });
        }

        effects.push(effect);

        self.set_effects(effects);
    }

    fn remove_effect(&mut self, effect: &LingeringEffect) {
        let mut effects = self.get_effects();

        if let LingeringEffectName::Stat(name) = &effect.name {
            let potency = match &effect.kind {
                LingeringEffectType::Buff => -effect.potency,
                LingeringEffectType::Debuff => effect.potency,
            };
            self.affect_stat(&BaseStatEffect {
                name: name.clone(),
                potency,
            })
        }

        effects.retain(|eff| eff != effect);

        self.set_effects(effects);
    }

    fn clear_effects(&mut self) {
        for effect in self.get_effects().iter() {
            self.remove_effect(effect)
        }
        self.set_effects(Vec::new())
    }

    fn apply_effects(&mut self) {
        let effects = self.get_effects();

        effects.iter().for_each(|effect| {
            match effect.name {
                LingeringEffectName::Poison => self.set_health(self.get_health() - effect.potency),
                LingeringEffectName::Regenerate => {
                    self.set_health(self.get_health() + effect.potency)
                }
                _ => (),
            }

            let mut new_effect = effect.clone();

            self.remove_effect(effect);

            if effect.duration != 1 {
                new_effect.duration -= 1;
                self.add_effect(new_effect);
            }
        })
    }
}
