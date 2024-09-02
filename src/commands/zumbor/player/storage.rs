use serde_json::Value;
use serenity::client::Context;

use crate::{
    commands::zumbor::{effects::LingeringEffect, player::stats::Stats},
    errors::Error,
    storage::StorageClient,
};

use super::Player;

// Fetches the player's save if it exists
pub async fn load_save(ctx: &Context, user_tag: &str) -> Result<Player, Error> {
    let data = ctx.data.read().await;

    let storage_client = data.get::<StorageClient>().unwrap();
    let path = "zumbor/saves/".to_string() + user_tag + ".json";

    let bytes = storage_client.get(&path).await?;

    let maybe_player: Result<Player, Error> = serde_json::from_slice(&bytes).map_err(Error::Json);

    // V2 players should be serializable straight to a struct
    if let Ok(player) = maybe_player {
        return Ok(player);
    }

    println!("Failed deserialise of object as struct");

    // Handles first versions of the Player object
    let maybe_player_map: Value = serde_json::from_slice(&bytes).unwrap();

    let name: String = maybe_player_map
        .get("name")
        .ok_or(Error::Plain("name field not present in data"))?
        .as_str()
        .expect("Name is a string")
        .into();
    let tag: String = maybe_player_map
        .get("user")
        .ok_or(Error::Plain("name field not present in data"))?
        .as_str()
        .expect("User is a string")
        .into();
    let description: String = maybe_player_map
        .get("description")
        .ok_or(Error::Plain("description field not present in data"))?
        .as_str()
        .expect("Description is a string")
        .into();
    let health: i16 = maybe_player_map
        .get("health")
        .ok_or(Error::Plain("health field not present in data"))?
        .as_u64()
        .expect("Health is a number")
        .try_into()
        .unwrap();
    let score: u16 = maybe_player_map
        .get("score")
        .ok_or(Error::Plain("score field not present in data"))?
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
        Err(Error::Plain("Stats should be an object / hash map"))
    }?;

    println!("Starting desrialise of effects..");
    let effects: Vec<LingeringEffect> =
        serde_json::from_value(maybe_player_map["effects"].clone()).unwrap_or(Vec::new());

    let player = Player {
        tag,
        name,
        description,
        health,
        score,
        stats,
        effects,
    };

    Ok(player)
}

impl Player {
    pub async fn delete_save(self, ctx: &Context) -> Result<(), Error> {
        let data = ctx.data.read().await;

        let storage_client = data
            .get::<StorageClient>()
            .ok_or(Error::Plain("Storage client not accessible!"))?;

        dbg!(&self.tag);
        storage_client
            .delete_json(("zumbor/saves/".to_string() + &self.tag).as_str())
            .await
    }

    pub async fn save(&self, ctx: &Context) -> Result<(), Error> {
        let data = ctx.data.read().await;

        let storage_client = data
            .get::<StorageClient>()
            .ok_or(Error::Plain("Storage client not accessible!"))?;

        let player_json: String = serde_json::to_string(&self).map_err(Error::Json)?;
        let save_name = "zumbor/saves/".to_string() + &self.tag;

        storage_client.create_json(&save_name, player_json).await
    }
}
