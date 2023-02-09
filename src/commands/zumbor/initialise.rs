use std::collections::HashMap;

use serenity::{
    model::prelude::{ActivityButton, Message, UserId},
    prelude::Context,
    Error,
};

use super::{
    display::{Display, DisplayBuilder},
    effects::{BaseEffect, Effectable},
    encounter::Encounter,
    player::Player,
};

pub async fn start(ctx: &Context, msg: &Message) -> Result<bool, Error> {
    let mut running_games: HashMap<UserId, bool> = HashMap::new();

    let display = DisplayBuilder::from(ctx, msg).await?;

    let player_id: UserId = msg.author.id;

    if running_games.get(&player_id).is_some() {
        display.say("You already have a Zumbor instance running dumbo.");
        return Err(Error::Other(
            "The user already has a Zumbor instance running",
        ));
    }

    running_games.insert(player_id, true);

    let mut player: Player = Player::load(player_id)
        .await
        .or(display.request_player().await)?;

    let display: Display = display.build()?;
    // initialise_player_events(player, display);

    loop {
        let mut _interaction: Option<ActivityButton>;

        let mut encounter: Encounter = Encounter::new().await?;

        display.encounter_details(&encounter).await?;

        let player_choice = match display.encounter_options(&encounter).await {
            Ok(val) => val,
            Err(err) => {
                println!("{}", err);
                display.say("Something went wrong retrieving the player choice");
                break Err(Error::Other("Player choice resulted in an error"));
            }
        };

        let encounter_option = encounter
            .options
            .get_mut(player_choice)
            .expect("Player choice is limited to encounter option keys");

        let player_roll = player.roll_stat(&encounter_option.stat);

        let encounter_result = if player_roll.value >= encounter_option.threshold.into() {
            &mut encounter_option.success
        } else {
            &mut encounter_option.fail
        };

        if let Some(effect) = &mut encounter_result.base_effect {
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
            player.add_effect(effect)
        }

        player.apply_effects();
        player.add_score(1);

        display.encounter_result(&encounter_result);

        if player.health <= 0 {
            player.effects.clear();

            display.say(format!("Uh oh {} died", &player.name).as_ref());

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
            player.remove();
            return Ok(true);
        }

        let resume_playing: bool = display.request_continue().await?;

        if resume_playing {
            continue;
        };

        display.say(format!("{} retires for now...", player.name).as_ref());

        match player.save().await {
            Ok(_saved) => display.say("Saved Successfully"),
            Err(err) => {
                println!("{}", err);
                display.say(
                    format!(
                        "Something went wrong while saving... Say goodbye to {}",
                        player.name
                    )
                    .as_ref(),
                );
            }
        }

        running_games.remove(&player_id);
    }
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
