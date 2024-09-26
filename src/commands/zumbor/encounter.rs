use crate::errors::Error;
use std::collections::HashMap;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serenity::{
    all::{Colour, CreateActionRow, CreateButton, CreateEmbed},
    prelude::Context,
};

use crate::StorageClient;

use super::{
    attributes::Attribute,
    effects::{
        BaseAttributeEffect, BaseEffect, BaseHealthEffect, LingeringEffect, LingeringEffectName,
        LingeringEffectType,
    },
    player::RollResult,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Encounter {
    pub title: String,
    pub text: String,
    pub color: Option<Colour>,
    pub options: HashMap<String, EncounterOption>,
}

impl Encounter {
    pub fn get_option(&mut self, name: &String) -> Option<&mut EncounterOption> {
        self.options.get_mut(name)
    }
}

impl From<&Encounter> for CreateEmbed {
    fn from(enc: &Encounter) -> CreateEmbed {
        CreateEmbed::default()
            .title(&enc.title)
            .description(&enc.text)
            .color(enc.color.unwrap_or_default())
    }
}

impl From<&Encounter> for CreateActionRow {
    fn from(enc: &Encounter) -> CreateActionRow {
        CreateActionRow::Buttons(
            enc.options
                .keys()
                .map(|key| CreateButton::new(key).label(key))
                .collect(),
        )
    }
}

impl TryFrom<Value> for Encounter {
    type Error = Error;

    fn try_from(enc: Value) -> Result<Self, Self::Error> {
        let mut options: HashMap<String, EncounterOption> = HashMap::new();

        if let Value::Object(options_map) = enc["options"].clone() {
            for (key, value) in options_map {
                let success_result = if let Value::Object(res) = value["Success"].clone() {
                    EncounterResult {
                        kind: EncounterResultKind::Success(
                            res["type"].as_str().unwrap().to_string(),
                        ),
                        title: res["title"].as_str().unwrap().into(),
                        text: res["text"].as_str().unwrap().into(),
                        base_effect: json_base_effect_to_struct(res.clone()),
                        lingering_effect: json_lingering_effect_to_struct(res),
                    }
                } else {
                    panic!("Success result should be an object");
                };

                let fail_result = if let Value::Object(res) = value["Fail"].clone() {
                    EncounterResult {
                        kind: EncounterResultKind::Fail(res["type"].as_str().unwrap().to_string()),
                        title: res["title"].as_str().unwrap().into(),
                        text: res["text"].as_str().unwrap().into(),
                        base_effect: json_base_effect_to_struct(res.clone()),
                        lingering_effect: json_lingering_effect_to_struct(res),
                    }
                } else {
                    panic!("Success result should be an object");
                };

                let option = EncounterOption {
                    threshold: value["threshold"].as_u64().unwrap().try_into().unwrap(),
                    stat: value["stat"].as_str().unwrap().try_into().unwrap(),
                    success: success_result,
                    fail: fail_result,
                };

                options.insert(key, option);
            }
        }

        let color = match &enc["color"] {
            Value::String(val) => hex_to_colour(val.as_ref()),
            _ => hex_to_colour("#000000"),
        };

        let encounter = Encounter {
            title: enc["title"].to_string(),
            text: enc["text"].to_string(),
            color: Some(color),
            options,
        };

        Ok(encounter)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncounterOption {
    pub threshold: u8,
    pub stat: Attribute,
    pub success: EncounterResult,
    pub fail: EncounterResult,
}

impl EncounterOption {
    pub fn test(&mut self, roll: &RollResult) -> &mut EncounterResult {
        match roll {
            RollResult::CriticalFail => &mut self.fail,
            RollResult::CriticalSuccess => &mut self.success,
            RollResult::Value(num) => {
                if *num >= self.threshold.into() {
                    &mut self.success
                } else {
                    &mut self.fail
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncounterResult {
    #[serde(alias = "type")]
    pub kind: EncounterResultKind,
    pub title: String,
    pub text: String,
    pub base_effect: Option<BaseEffect>,
    pub lingering_effect: Option<LingeringEffect>,
}

impl From<&EncounterResult> for CreateEmbed {
    fn from(result: &EncounterResult) -> CreateEmbed {
        let embed = CreateEmbed::default()
            .title(&result.title)
            .description(&result.text)
            .colour(match &result.kind {
                EncounterResultKind::Success(_) => Colour::from((20, 240, 60)),
                EncounterResultKind::Fail(_) => Colour::from((240, 40, 20)),
            });

        if let Some(effect) = &result.base_effect {
            match effect {
                BaseEffect::Attribute(effect) => {
                    embed.field(effect.name.clone(), effect.potency.to_string(), true)
                }
                BaseEffect::Health(effect) => {
                    embed.field("Health", effect.potency.to_string(), true)
                }
            }
        } else {
            embed
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EncounterResultKind {
    Success(String),
    Fail(String),
}

// Return a random encounter from the storage bucket
pub async fn get_random(ctx: &Context) -> Result<Encounter, Error> {
    let data = ctx.data.read().await;

    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in context");

    let objects = storage_client.get_objects("zumbor/encounters").await?;

    // println!("{:?}", objects);

    let object = objects
        .choose(&mut rand::thread_rng())
        .expect("Random number is limited to the number of objects");

    let byte_array = storage_client.get(&object.name).await?;

    let encounter: Result<Encounter, _> = serde_json::from_slice(&byte_array);

    // V2 encounters should be serializable straight to a struct
    if let Ok(encounter) = encounter {
        println!("Serialize success for {}", encounter.title);
        return Ok(encounter);
    } else {
        println!("Failed to deserialize {:?}", encounter);
    }

    // Handles previous versions of the Encounter object
    let enc: Value = serde_json::from_slice(&byte_array).unwrap();

    let encounter = Encounter::try_from(enc)?;

    let mut encounter_name = object
        .name
        .as_str()
        .replace(" ", "-")
        .replace("%20", "-")
        .replace("/encounters/", "/encounters/v2/");

    encounter_name.make_ascii_lowercase();

    let encounter_json: String = serde_json::to_string(&encounter).map_err(Error::from)?;

    storage_client
        .create_json(&encounter_name, encounter_json)
        .await?;

    Ok(encounter)
}

fn hex_to_colour(hex: &str) -> Colour {
    let hex_str = &hex[1..];
    Colour::from(u64::from_str_radix(hex_str, 16).expect("Incorrect format"))
}

fn json_base_effect_to_struct(map: Map<String, Value>) -> Option<BaseEffect> {
    let effect = map
        .get("baseEffect")
        .map(|val| val.as_object().expect("baseEffect to be an object"));

    effect.and_then(
        |effect| match effect["name"].as_str().expect("name is a string") {
            "Heal" => Some(BaseEffect::Health(BaseHealthEffect {
                potency: effect["potency"].as_u64().unwrap().try_into().unwrap(),
            })),
            "Damage" => Some(BaseEffect::Health(BaseHealthEffect {
                potency: -<i64 as TryInto<i16>>::try_into(effect["potency"].as_i64().unwrap())
                    .unwrap(),
            })),
            _ => effect["name"]
                .as_str()
                .expect("Name to be a string")
                .try_into()
                .map(|name| {
                    BaseEffect::Attribute(BaseAttributeEffect {
                        name,
                        potency: effect["potency"].as_i64().unwrap().try_into().unwrap(),
                    })
                })
                .ok(),
        },
    )
}

fn json_lingering_effect_to_struct(map: Map<String, Value>) -> Option<LingeringEffect> {
    let effect = map
        .get("additionalEffect")
        .map(|val| val.as_object().expect("additionalEffect to be an object"));

    println!("{:?}", effect);

    effect.and_then(
        |effect| match effect["name"].as_str().expect("name is a string") {
            "Poison" => Some(LingeringEffect {
                kind: LingeringEffectType::Debuff,
                name: LingeringEffectName::Poison,
                duration: effect["duration"]
                    .as_u64()
                    .expect("duration to be a number")
                    .try_into()
                    .expect("Should be small enough to convert to a 16 bit signed integer"),
                potency: effect["potency"]
                    .as_u64()
                    .expect("potency to be a number")
                    .try_into()
                    .expect("Should be small enough to convert to a 16 bit signed integer"),
            }),
            "Regenerate" => Some(LingeringEffect {
                kind: LingeringEffectType::Debuff,
                name: LingeringEffectName::Poison,
                duration: effect["duration"]
                    .as_u64()
                    .expect("duration to be a number")
                    .try_into()
                    .expect("Should be small enough to convert to a 16 bit signed integer"),
                potency: effect["potency"]
                    .as_u64()
                    .expect("potency to be a number")
                    .try_into()
                    .expect("Should be small enough to convert to a 16 bit signed integer"),
            }),
            _ => (effect["name"].as_str().expect("Name to be a string"))
                .try_into()
                .map(|name| LingeringEffect {
                    kind: LingeringEffectType::try_from(
                        effect["type"].as_str().expect("type to be a string"),
                    )
                    .unwrap_or_else(|err| {
                        println!("Using deafult buff type. {}", err);
                        LingeringEffectType::Buff
                    }),
                    name: LingeringEffectName::Stat(name),
                    duration: effect["duration"]
                        .as_u64()
                        .expect("duration to be a number")
                        .try_into()
                        .expect("Should be small enough to convert to a 16 bit signed integer"),
                    potency: effect["potency"]
                        .as_u64()
                        .expect("potency to be a number")
                        .try_into()
                        .expect("Should be small enough to convert to a 16 bit signed integer"),
                })
                .ok(),
        },
    )
}
