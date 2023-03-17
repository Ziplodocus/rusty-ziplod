use std::{collections::HashMap, fmt, sync::Arc, time::Duration};

use serenity::{
    builder::CreateEmbed,
    futures::lock::{Mutex, MutexGuard},
    http::Http,
    model::prelude::{
        interaction::message_component::MessageComponentInteraction, ActivityButton, ChannelId,
        Message, UserId,
    },
    prelude::Context,
    Error,
};
use tokio::sync::broadcast::error::TryRecvError;

use crate::StorageClient;

use super::{
    display::{ContinueOption, Display},
    effects::{BaseEffect, Effectable, LingeringEffect},
    encounter::{self, Encounter},
    player::Player,
};

pub async fn start(ctx: &Context, msg: &Message) -> Result<bool, Error> {
    let mut running_games: HashMap<UserId, bool> = HashMap::new();

    let player_id: UserId = msg.author.id;

    if running_games.get(&player_id).is_some() {
        msg.channel_id
            .say(ctx, "You already have a Zumbor instance running you mug")
            .await?;
        return Err(Error::Other(
            "The user already has a Zumbor instance running",
        ));
    }

    running_games.insert(player_id, true);

    let player: Player = Player::load(player_id).await.or(request_player().await)?;

    let player_mutex: Mutex<Player> = Mutex::new(player);

    let mut display: Display = Display::builder()
        .channel(msg.channel_id)
        .user(player_id)
        .context(ctx)
        .player(&player_mutex)
        .build();

    loop {
        let mut encounter: Encounter = Encounter::new(); //encounter::get(ctx).await?;

        let (player_choice, current_message) = display.encounter_details(&encounter).await?;

        let encounter_option = encounter
            .options
            .get_mut(&player_choice)
            .expect("Player choice should be limited to encounter option keys");

        let mut player: MutexGuard<Player> = player_mutex.lock().await;
        let player_roll = player.roll_stat(&encounter_option.stat);

        let encounter_result = if player_roll.value >= encounter_option.threshold.into() {
            &mut encounter_option.success
        } else {
            &mut encounter_option.fail
        };

        // dbg!(encounter_result.clone());

        if let Some(effect) = &mut encounter_result.base_effect {
            // dbg!(effect.clone());
            match effect {
                BaseEffect::Stat(eff) => {
                    if player_roll.critical {
                        eff.potency *= 2;
                    }
                }
                BaseEffect::Health(eff) => {
                    if player_roll.critical {
                        eff.potency *= 2;
                    }
                }
            }
            player.affect(effect)
        }

        if let Some(effect) = encounter_result.lingering_effect.clone() {
            println!("Added lingering effect: {}", effect.name);
            display.queue_message(effect.clone().into());
            player.add_effect(effect)
        }

        for effect in player.get_effects() {
            if effect.duration == 1 {
                let mut end_embed: CreateEmbed = effect.clone().into();
                end_embed.title(format!("{} {} has expired", effect.name, effect.kind));
                display.queue_message(end_embed);
            }
        }
        player.apply_effects();
        player.add_score(1);

        drop(player);

        if let Err(err) = display
            .encounter_result(&encounter_result, current_message)
            .await
        {
            println!("Unable to display the encounter result. {}", err);
        }

        let mut player: MutexGuard<Player> = player_mutex.lock().await;

        if player.health <= 0 {
            player.effects.clear();

            display
                .say(format!("Uh oh {} died", &player.name).as_ref())
                .await;

            // match scoreboard.update(player).await {
            //     Ok(did_win) => match did_win {
            //         true => display.say("You win, you winner!"),
            //         false => display.say("You lose, loser!"),
            //     },
            //     Err(err) => {
            //         display.say("Fetching scoreboard failed!");
            //         println!("{}", err)
            //     }
            // };

            running_games.remove(&player_id);
            // let result = player.remove();
            // if let Err(err) = result.await {
            //     println!("{}", err);
            //     println!("Unable to remove player");
            // }
            return Ok(true);
        }

        drop(player);

        let resume_playing: ContinueOption = display.request_continue().await?;

        if let ContinueOption::Continue = resume_playing {
            continue;
        };

        let player: MutexGuard<Player> = player_mutex.lock().await;

        nice_message(
            ctx,
            msg.channel_id,
            "Resting...".to_owned(),
            player.name.clone() + " takes a break",
        )
        .await?;

        // match player.save().await {
        //     Ok(_saved) => display.say("Saved Successfully").await,
        //     Err(err) => {
        //         println!("{}", err);
        //         display
        //             .say(
        //                 format!(
        //                     "Something went wrong while saving... Say goodbye to {}",
        //                     player.name
        //                 )
        //                 .as_ref(),
        //             )
        //             .await;
        //     }
        // }

        running_games.remove(&player_id);

        break;
    }
    Ok(true)
}

// fn initialise_player_events(player: Player, display: Display) {
//     player.on(PlayerEvent::EffectStart, |&player, effect: Effect| {
//         display.queue_embed(format!(
//             "{} has received a {} {}",
//             player.name, effect.name, effect.typ
//         ));
//     });
//     player.on(PlayerEvent::EffectEnd, |&player, effect: Effect| {
//         display.queue_embed(format!(
//             "{}'s {} {} has ended",
//             player.name, effect.name, effect.typ
//         ));
//     });
//     player.on(PlayerEvent::EffectApplied, |&player, effect: Effect| {
//         display.queue_embed(format!(
//             "{} has received a {} {}",
//             player.name, effect.name, effect.typ
//         ));
//     });
// }

async fn request_player() -> Result<Player, Error> {
    let player: Player = serde_json::from_str(
        "{
            user: {
                0: 1
            },
            description: 'Really good looking',
            name: 'Handsome Jack',
            health: '20',
            score: '0',
            stats: {
                charisma: '10',
                strength: '3',
                wisdom: '2',
                agility: '1',
            },
            effects: [],
        }",
    )?;
    Ok(player)
}

async fn nice_message(
    ctx: impl AsRef<Http>,
    channel_id: ChannelId,
    title: String,
    description: String,
) -> Result<Message, Error> {
    channel_id
        .send_message(ctx, |msg| {
            msg.embed(|emb| emb.title(title).description(description))
        })
        .await
}
