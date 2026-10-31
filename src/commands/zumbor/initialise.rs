use serenity::{
    all::CreateMessage,
    builder::CreateEmbed,
    model::{
        prelude::{ChannelId, Message, UserId},
        user::User,
    },
};

use crate::{Context, errors::Error};

use super::{
    effects::Effectable,
    encounter::{self, Encounter},
    player::{self, RollResult},
    ui::{ContinueOption, UI},
};

pub async fn start(ctx: Context<'_>) -> Result<bool, Error> {
    let user: &User = ctx.author();
    let zumbor = &(ctx.data().zumbor);

    if let Err(err) = zumbor.instances.write().await.add(user.id) {
        nice_message(
            ctx,
            "You Fail!".to_string(),
            "You already have a running instance of Zumbor you fool!".to_string(),
        )
        .await
        .expect("To send a message");
        return Err(err);
    };

    let mut player = if let Ok(player) = player::storage::load_save(ctx, &user.tag()).await {
        player
    } else {
        player::create(ctx, user).await?
    };

    let mut ui = UI::new(ctx);

    loop {
        let mut encounter: Encounter = encounter::get_random(ctx).await?;

        let (player_choice, current_message) = ui.encounter_details(&encounter, &player).await?;

        let encounter_option = encounter
            .get_option(&player_choice)
            .expect("Player choice should be limited to encounter option keys");

        let player_roll = player.roll_stat(&encounter_option.stat);

        let encounter_result = encounter_option.test(&player_roll);

        // Handle base effect of the result
        if let Some(effect) = &mut encounter_result.base_effect {
            match player_roll {
                RollResult::CriticalFail | RollResult::CriticalSuccess => {
                    effect.set_potency(effect.get_potency() * 2);
                }
                _ => (),
            };
            player.affect(effect)
        }

        // Handle lingering effects of the result
        if let Some(effect) = &encounter_result.lingering_effect {
            println!("Added lingering effect: {}", effect.name);
            let gain_embed: CreateEmbed = effect.into();

            ui.queue_message(
                gain_embed.title(format!("Received a {} {}", effect.name, effect.kind)),
            );

            player.add_effect(effect.clone())
        }

        for effect in player.get_effects() {
            if effect.duration == 1 {
                ui.queue_message(CreateEmbed::new().title(format!(
                    "A potency {} {} {} has expired",
                    effect.potency, effect.name, effect.kind
                )));
            }
        }

        player.apply_effects();
        player.add_score(1);

        if let Err(err) = ui
            .encounter_result(encounter_result, &player, current_message)
            .await
        {
            println!("Unable to display the encounter result. {}", err);
        }

        if player.health <= 0 {
            player.effects.clear();

            ui.say(format!("Uh oh {} died", &player.name).as_ref())
                .await;

            if let Err(err) = player.delete_save(ctx).await {
                println!("Unable to delete the player save. {}", err);
            };

            // match scoreboard.update(player).await {
            //     Ok(did_win) => match did_win {
            //         true => ui.say("You win, you winner!"),
            //         false => ui.say("You lose, loser!"),
            //     },
            //     Err(err) => {
            //         ui.say("Fetching scoreboard failed!");
            //         println!("{}", err)
            //     }
            // };

            zumbor.instances.write().await.remove(user.id);
            return Ok(true);
        }

        let resume_playing: ContinueOption = ui.request_continue(&player).await?;

        if let ContinueOption::Continue = resume_playing {
            continue;
        };

        zumbor.instances.write().await.remove(user.id);

        ui.queue_message(
            CreateEmbed::new()
                .title("Resting...".to_owned())
                .description(player.name.clone() + " takes a break"),
        );

        match player.save(ctx).await {
            Ok(_saved) => ui.queue_message(CreateEmbed::new().title("Save Succesful.")),
            Err(err) => {
                println!("{}", err);
                ui.queue_message(
                    CreateEmbed::new()
                        .title("Ruh Roh Wraggy...".to_string())
                        .description(format!(
                            "Something went wrong while saving... Say goodbye to {}",
                            player.name
                        )),
                )
            }
        }

        ui.send_messages().await.unwrap();

        break;
    }
    Ok(true)
}

// fn initialise_player_events(player: Player, ui: Display) {
//     player.on(PlayerEvent::EffectStart, |&player, effect: Effect| {
//         ui.queue_embed(format!(
//             "{} has received a {} {}",
//             player.name, effect.name, effect.typ
//         ));
//     });
//     player.on(PlayerEvent::EffectEnd, |&player, effect: Effect| {
//         ui.queue_embed(format!(
//             "{}'s {} {} has ended",
//             player.name, effect.name, effect.typ
//         ));
//     });
//     player.on(PlayerEvent::EffectApplied, |&player, effect: Effect| {
//         ui.queue_embed(format!(
//             "{} has received a {} {}",
//             player.name, effect.name, effect.typ
//         ));
//     });
// }

// fn request_player<'a>(user: &User) -> Result<Player, Error> {
//     Ok(Player {
//         tag: user.tag(),
//         description: "Really good looking".to_string(),
//         name: "Handsome Jack".to_string(),
//         health: 20,
//         score: 0,
//         stats: Stats {
//             charisma: 10,
//             strength: 3,
//             wisdom: 2,
//             agility: 1,
//         },
//         effects: Vec::new(),
//     })
// }

async fn nice_message(
    ctx: Context<'_>,
    title: String,
    description: String,
) -> Result<Message, Error> {
    let res = ctx
        .channel_id()
        .send_message(
            ctx,
            CreateMessage::new().embed(CreateEmbed::new().title(title).description(description)),
        )
        .await?;
    Ok(res)
}
