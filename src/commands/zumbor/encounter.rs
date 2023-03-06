use std::{collections::HashMap, sync::Arc};

use google_cloud_storage::{
    client::Client,
    http::objects::{download::Range, get::GetObjectRequest, list::ListObjectsRequest},
};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::{Error as JsonError, Value};
use serenity::{
    builder::CreateEmbed, futures::lock::MutexGuard, prelude::Context, utils::Colour, Error,
};

use crate::{commands::zumbor::effects::map_attribute_name, StorageClient};

use super::effects::{
    Attribute, BaseEffect, BaseHealthEffect, BaseStatEffect, LingeringEffect, LingeringEffectName,
    LingeringEffectType,
};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EncounterResultName {
    Success(String),
    Fail(String),
}

// Return a random encounter from the storage bucket
pub async fn get(ctx: &Context) -> Result<Encounter, JsonError> {
    let data = ctx.data.read().await;

    let storage_client = data.get::<StorageClient>().unwrap();

    let client = &storage_client.client;

    let list_request = ListObjectsRequest {
        bucket: "ziplod-assets".into(),
        prefix: Some("zumbor/encounters".into()),
        ..ListObjectsRequest::default()
    };

    let list = client
        .list_objects(&list_request, None)
        .await
        .unwrap()
        .items
        .unwrap();

    let object = list.choose(&mut rand::thread_rng()).unwrap();

    let request = GetObjectRequest {
        bucket: "ziplod-assets".into(),
        object: object.name.clone(),
        ..Default::default()
    };

    let range = Range::default();

    let byte_array = client
        .download_object(&request, &range, None)
        .await
        .unwrap();

    let encounter: Result<Encounter, _> = serde_json::from_slice(&byte_array);

    if let Ok(encounter) = encounter {
        return Ok(encounter);
    }

    let enc: Value = serde_json::from_slice(&byte_array).unwrap();

    let mut options: HashMap<String, EncounterOption> = HashMap::new();

    if let Value::Object(options_map) = enc["options"].clone() {
        for (key, value) in options_map {
            dbg!(value.clone());

            let success_result = if let Value::Object(res) = value["Success"].clone() {
                let effect = res["baseEffect"]
                    .as_object()
                    .expect("result to have a baseEffect");

                let base_effect = match effect["name"].as_str().expect("name is a string") {
                    "Heal" => Some(BaseEffect::Health(BaseHealthEffect {
                        potency: effect["potency"].as_u64().unwrap().try_into().unwrap(),
                    })),
                    "Damage" => Some(BaseEffect::Health(BaseHealthEffect {
                        potency: effect["potency"].as_u64().unwrap().try_into().unwrap(),
                    })),
                    _ => match map_attribute_name(
                        effect["name"].as_str().expect("Name to be a string"),
                    ) {
                        Some(name) => Some(BaseEffect::Stat(BaseStatEffect {
                            name,
                            potency: effect["potency"].as_i64().unwrap().try_into().unwrap(),
                        })),
                        None => None,
                    },
                };

                EncounterResult {
                    kind: EncounterResultName::Success(res["type"].as_str().unwrap().to_string()),
                    title: res["title"].as_str().unwrap().into(),
                    text: res["text"].as_str().unwrap().into(),
                    base_effect,
                    lingering_effect: None,
                }
            } else {
                panic!("Success result should be an object");
            };

            let fail_result = if let Value::Object(res) = value["Fail"].clone() {
                EncounterResult {
                    kind: EncounterResultName::Fail(res["type"].as_str().unwrap().to_string()),
                    title: res["title"].as_str().unwrap().into(),
                    text: res["text"].as_str().unwrap().into(),
                    base_effect: None,
                    lingering_effect: None,
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

    let encounter = Encounter {
        title: enc["title"].to_string(),
        text: enc["text"].to_string(),
        color: Some(Colour::BLURPLE),
        options,
    };

    dbg!(encounter);

    panic!();
    Ok(encounter)

    // encounter
    // client.download_object(&GetObjectRequest {
    //     bucket: "ziplod-assets",
    // })
}
