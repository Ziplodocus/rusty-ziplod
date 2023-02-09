use serde::{Deserialize, Serialize};

use super::player::Stats;

#[derive(Serialize, Deserialize)]
pub enum BaseEffect {
    Stat(BaseStatEffect),
    Health(BaseHealthEffect),
}

#[derive(Serialize, Deserialize)]
pub struct BaseHealthEffect {
    pub potency: i16,
}

#[derive(Serialize, Deserialize)]
pub struct BaseStatEffect {
    pub name: Attribute,
    pub potency: i16,
}

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum Attribute {
    Charisma,
    Strength,
    Wisdom,
    Agility,
}

fn map_attribute_name(potential_attribute_name: &str) -> Option<Attribute> {
    match potential_attribute_name {
        "Charisma" | "charisma" => Some(Attribute::Charisma),
        "Strength" | "strength" => Some(Attribute::Strength),
        "Agility" | "agility" => Some(Attribute::Agility),
        "Wisdom" | "wisdom" => Some(Attribute::Wisdom),
        _ => None,
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub struct LingeringEffect {
    kind: LingeringEffectType,
    name: LingeringEffectName,
    potency: i16,
    duration: i16,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum LingeringEffectName {
    Stat(Attribute),
    Poison,
    Regenerate,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum LingeringEffectType {
    Buff,
    Debuff,
}

pub trait Effectable {
    fn get_effects(&self) -> Vec<LingeringEffect>;
    fn set_effects(&mut self, effects: Vec<LingeringEffect>);
    fn get_health(&self) -> i16;
    fn set_health(&mut self, health: i16);
    fn get_stats(&self) -> Stats;
    fn set_stats(&mut self, stats: Stats);

    fn affect(&mut self, effect: &BaseEffect) {
        match effect {
            BaseEffect::Health(eff) => self.affect_health(eff),
            BaseEffect::Stat(eff) => self.affect_stat(eff),
        }
    }

    fn affect_health(&mut self, effect: &BaseHealthEffect) {
        self.set_health(self.get_health() + effect.potency);
    }

    fn affect_stat(&mut self, effect: &BaseStatEffect) {
        let mut stats = self.get_stats();
        let stat = stats.get_mut(effect.name.clone());
        *stat += effect.potency;
        self.set_stats(stats);
    }

    fn add_effect(&mut self, effect: LingeringEffect) {
        let mut effects = self.get_effects();

        match &effect.name {
            LingeringEffectName::Stat(attr) => self.affect_stat(&BaseStatEffect {
                name: attr.clone(),
                potency: effect.potency,
            }),
            _ => (),
        }

        effects.push(effect);

        self.set_effects(effects);
    }

    fn remove_effect(&mut self, effect: &LingeringEffect) {
        let mut effects = self.get_effects();

        match &effect.name {
            LingeringEffectName::Stat(name) => {
                let potency = match &effect.kind {
                    LingeringEffectType::Buff => -effect.potency,
                    LingeringEffectType::Debuff => effect.potency,
                };
                self.affect_stat(&BaseStatEffect {
                    name: name.clone(),
                    potency,
                })
            }
            _ => (),
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
