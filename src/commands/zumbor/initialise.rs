use std::{collections::HashMap, fmt, sync::Arc, time::Duration};

use serenity::{
    builder::CreateEmbed,
    futures::lock::{Mutex, MutexGuard},
    http::Http,
    model::{
        prelude::{ChannelId, Message, UserId},
        user::User,
    },
    prelude::Context,
    utils::Colour,
    Error,
};

use crate::ZumborInstances;

use super::{
    display::{ContinueOption, Display},
    effects::{BaseEffect, Effectable},
    encounter::Encounter,
    player::{self, Player, Stats},
};

pub async fn start(ctx: &Context, msg: &Message) -> Result<bool, Error> {
    // let mut running_games: HashMap<UserId, bool> = HashMap::new();

    let user: &User = &msg.author;

    if let Err(err) = add_player_to_instance(ctx, user.id).await {
        nice_message(
            ctx,
            msg.channel_id,
            "You Fail!".to_string(),
            "You already have a running instance of Zumbor you fool!".to_string(),
        )
        .await
        .unwrap();
        return Err(err)
    };

    let player: Player = player::get(ctx, user.tag())
        .await
        .or(request_player(user))?;

    let player_mutex: Mutex<Player> = Mutex::new(player);

    let mut display: Display = Display::builder()
        .channel(msg.channel_id)
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
            let mut gain_embed: CreateEmbed = effect.clone().into();
            gain_embed.title(format!("Received a {} {}", effect.name, effect.kind));
            gain_embed.colour::<Colour>(effect.name.clone().into());
            display.queue_message(gain_embed);
            player.add_effect(effect)
        }

        for effect in player.get_effects() {
            if effect.duration == 1 {
                let end_embed = quick_embed(
                    format!(
                        "A potency {} {} {} has expired",
                        effect.potency, effect.name, effect.kind
                    ),
                    None,
                );
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

            remove_player_from_instance(ctx, user.id).await;
            return Ok(true);
        }

        drop(player);

        let resume_playing: ContinueOption = display.request_continue().await?;

        if let ContinueOption::Continue = resume_playing {
            continue;
        };

        remove_player_from_instance(ctx, user.id).await;

        {
            let mut data = ctx.data.write().await;
            let zumbor = data.get_mut::<ZumborInstances>().unwrap();
            if zumbor.instances.contains(&user.id) {
                nice_message(
                    ctx,
                    msg.channel_id,
                    "You're already playing...".to_string(),
                    "".to_string(),
                )
                .await
                .expect("To be able to send a message without panic");
                return Err(Error::Other(
                    "The user currently has an active Zumbor instance",
                ));
            }
            zumbor.instances.push(user.id);
        }

        let player: MutexGuard<Player> = player_mutex.lock().await;

        display.queue_message(quick_embed(
            "Resting...".to_owned(),
            Some(player.name.clone() + " takes a break"),
        ));

        match player::save(ctx, &player).await {
            Ok(_saved) => {
                display.queue_message(quick_embed(format!("Save Succesful."), Some(format!(""))))
            }
            Err(err) => {
                println!("{}", err);
                display.queue_message(quick_embed(
                    "Ruh Roh Wraggy...".to_string(),
                    Some(format!(
                        "Something went wrong while saving... Say goodbye to {}",
                        player.name
                    )),
                ))
            }
        }

        display.send_messages().await.unwrap();

        // running_games.remove(&user.id);

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

fn request_player<'a>(user: &User) -> Result<Player, Error> {
    Ok(Player {
        tag: user.tag(),
        description: "Really good looking".to_string(),
        name: "Handsome Jack".to_string(),
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

fn quick_embed(title: String, description: Option<String>) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title(title);
    if let Some(description) = description {
        embed.description(description);
    }

    embed
}

/**
 * Adds the user ID to the running instances saved into th context, returns an error if the user id exists in the intances vec already
 */
async fn add_player_to_instance(ctx: &Context, user_id: UserId) -> Result<bool, Error> {
    let mut data = ctx.data.write().await;
    let zumbor = data.get_mut::<ZumborInstances>().unwrap();
    if zumbor.instances.contains(&user_id) {
        Err(Error::Other(
            "The user currently has an active Zumbor instance",
        ))
    } else {
        zumbor.instances.push(user_id);
        Ok(true)
    }
}

async fn remove_player_from_instance(ctx: &Context, user_id: UserId) {
    let mut data = ctx.data.write().await;
    let zumbor = data.get_mut::<ZumborInstances>().unwrap();
    // Ignore if no such element is found
    if let Some(pos) = zumbor.instances.iter().position(|x| *x == user_id) {
        zumbor.instances.remove(pos);
    }
}
