use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serenity::{utils::Colour, Error};

use super::effects::{Attribute, BaseEffect, LingeringEffect};

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
                base_effect: None,
                lingering_effect: None,
            },
            fail: EncounterResult {
                kind: EncounterResultName::Success("Oh no".into()),
                title: "You gone fucked up".into(),
                text: "But nothing bad has happened".into(),
                base_effect: None,
                lingering_effect: None,
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EncounterResult {
    pub kind: EncounterResultName,
    pub title: String,
    pub text: String,
    pub base_effect: Option<BaseEffect>,
    pub lingering_effect: Option<LingeringEffect>,
}

#[derive(Serialize, Deserialize)]
pub enum EncounterResultName {
    Success(String),
    Fail(String),
}
