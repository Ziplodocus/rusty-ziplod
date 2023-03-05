use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serenity::{builder::CreateEmbed, utils::Colour, Error};

use super::effects::{
    Attribute, BaseEffect, BaseHealthEffect, BaseStatEffect, LingeringEffect, LingeringEffectName,
    LingeringEffectType,
};

#[derive(Serialize, Deserialize)]
pub struct Encounter {
    pub title: String,
    pub text: String,
    pub color: Option<Colour>,
    pub options: HashMap<String, EncounterOption>,
}

impl Encounter {
    pub async fn new() -> Result<Self, Error> {
        let mut options = HashMap::new();
        options.insert("Win".into(), EncounterOption::default());

        Ok(Encounter {
            title: "hey".to_owned(),
            text: "yeah".to_owned(),
            color: None,
            options,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct EncounterOption {
    pub threshold: u8,
    pub stat: Attribute,
    pub success: EncounterResult,
    pub fail: EncounterResult,
}

impl Default for EncounterOption {
    fn default() -> EncounterOption {
        EncounterOption {
            threshold: 10,
            stat: Attribute::Strength,
            success: EncounterResult {
                kind: EncounterResultName::Success("Oh no".into()),
                title: "You gone fucked up".into(),
                text: "But nothing bad has happened".into(),
                base_effect: Some(BaseEffect::Stat(BaseStatEffect {
                    name: Attribute::Charisma,
                    potency: 2,
                })),
                lingering_effect: None, // Some(LingeringEffect {
                                        //     kind: LingeringEffectType::Buff,
                                        //     name: LingeringEffectName::Stat(Attribute::Strength),
                                        //     potency: 5,
                                        //     duration: 4,
                                        // }),
            },
            fail: EncounterResult {
                kind: EncounterResultName::Success("Oh no".into()),
                title: "You gone fucked up".into(),
                text: "But something bad has happened".into(),
                base_effect: Some(BaseEffect::Health(BaseHealthEffect { potency: 5 })),
                lingering_effect: None, // Some(LingeringEffect {
                                        //     kind: LingeringEffectType::Buff,
                                        //     name: LingeringEffectName::Poison,
                                        //     potency: 5,
                                        //     duration: 4,
                                        // }),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EncounterResult {
    pub kind: EncounterResultName,
    pub title: String,
    pub text: String,
    pub base_effect: Option<BaseEffect>,
    pub lingering_effect: Option<LingeringEffect>,
}

impl From<EncounterResult> for CreateEmbed {
    fn from(result: EncounterResult) -> CreateEmbed {
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
}

#[derive(Serialize, Deserialize, Clone)]
pub enum EncounterResultName {
    Success(String),
    Fail(String),
}
