use crate::errors::Error;
use std::{collections::HashMap, future};

use cloud_storage::ListRequest;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serenity::{
    builder::{CreateComponents, CreateEmbed},
    futures::StreamExt,
    model::prelude::component::ButtonStyle,
    prelude::Context,
    utils::Colour,
};

use crate::{commands::zumbor::effects::map_attribute_name, StorageClient};

use super::{
    effects::{
        Attribute, BaseEffect, BaseHealthEffect, BaseStatEffect, LingeringEffect,
        LingeringEffectName, LingeringEffectType,
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
        let mut embed = CreateEmbed::default();
        embed
            .title(&enc.title)
            .description(&enc.text)
            .color(enc.color.unwrap_or_default());
        embed
    }
}

impl From<&Encounter> for CreateComponents {
    fn from(enc: &Encounter) -> CreateComponents {
        let mut components = CreateComponents::default();
        components.create_action_row(|row| {
            enc.options.iter().for_each(|(label, _)| {
                row.create_button(|but| {
                    but.custom_id(&label)
                        .label(&label)
                        .style(ButtonStyle::Primary)
                });
            });
            row
        });
        components
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
    pub kind: EncounterResultName,
    pub title: String,
    pub text: String,
    pub base_effect: Option<BaseEffect>,
    pub lingering_effect: Option<LingeringEffect>,
}

impl From<&EncounterResult> for CreateEmbed {
    fn from(result: &EncounterResult) -> CreateEmbed {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EncounterResultName {
    Success(String),
    Fail(String),
}

// Return a random encounter from the storage bucket
pub async fn fetch(ctx: &Context) -> Result<Encounter, Error> {
    let data = ctx.data.read().await;

    let storage_client = data
        .get::<StorageClient>()
        .expect("Storage client is available in context");

    let client = &storage_client.client;

    let list = client
        .object()
        .list(
            &storage_client.bucket_name,
            ListRequest {
                prefix: Some("zumbor/encounters".to_owned()),
                max_results: Some(1000),
                ..Default::default()
            },
        )
        .await?;

    let mut objects = vec![];
    list.for_each(|list| {
        let mut items = list.unwrap().items;
        objects.append(&mut items);
        future::ready(())
    })
    .await;

    println!("{:?}", objects);

    let object = objects.choose(&mut rand::thread_rng()).unwrap();

    let byte_array = storage_client.download(&object.name).await?;

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

    let mut options: HashMap<String, EncounterOption> = HashMap::new();

    if let Value::Object(options_map) = enc["options"].clone() {
        for (key, value) in options_map {
            let success_result = if let Value::Object(res) = value["Success"].clone() {
                EncounterResult {
                    kind: EncounterResultName::Success(res["type"].as_str().unwrap().to_string()),
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
                    kind: EncounterResultName::Fail(res["type"].as_str().unwrap().to_string()),
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
                stat: map_attribute_name(value["stat"].as_str().unwrap()).unwrap(),
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

    let encounter_name = object
        .name
        .as_str()
        .replace("/encounters/", "/encounters/v2/");

    let encounter_json: String =
        serde_json::to_string(&encounter).map_err(|err| Error::from(err))?;

    storage_client
        .upload_json(&encounter_name, encounter_json)
        .await?;

    // client.upload();

    // let upload_request = UploadObjectRequest {
    //     bucket: "ziplod-assets".into(),
    //     ..Default::default()
    // };

    // let upload_media = Media::new(encounter_name);

    // client
    //     .upload_object(
    //         &upload_request,
    //         encounter_json,
    //         &UploadType::Simple(upload_media),
    //     )
    //     .await
    //     .map_err(|err| {
    //         dbg!(err);
    //         Error::Other("Failed to upload object")
    //     })?;

    Ok(encounter)
}

fn hex_to_colour(hex: &str) -> Colour {
    let hex_str = &hex[1..];
    Colour::from(u64::from_str_radix(hex_str, 16).expect("Incorrect format"))
}

fn json_base_effect_to_struct(map: Map<String, Value>) -> Option<BaseEffect> {
    let effect = map
        .get("baseEffect")
        .and_then(|val| Some(val.as_object().expect("baseEffect to be an object")));

    effect.and_then(
        |effect| match effect["name"].as_str().expect("name is a string") {
            "Heal" => Some(BaseEffect::Health(BaseHealthEffect {
                potency: effect["potency"].as_u64().unwrap().try_into().unwrap(),
            })),
            "Damage" => Some(BaseEffect::Health(BaseHealthEffect {
                potency: -1
                    * <i64 as TryInto<i16>>::try_into(effect["potency"].as_i64().unwrap()).unwrap(),
            })),
            _ => match map_attribute_name(effect["name"].as_str().expect("Name to be a string")) {
                Some(name) => Some(BaseEffect::Stat(BaseStatEffect {
                    name,
                    potency: effect["potency"].as_i64().unwrap().try_into().unwrap(),
                })),
                None => None,
            },
        },
    )
}

fn json_lingering_effect_to_struct(map: Map<String, Value>) -> Option<LingeringEffect> {
    let effect = map
        .get("addtionalEffect")
        .and_then(|val| Some(val.as_object().expect("additionalEffect to be an object")));

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
            _ => map_attribute_name(effect["name"].as_str().expect("Name to be a string")).map(
                |name| LingeringEffect {
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
                },
            ),
        },
    )
}
