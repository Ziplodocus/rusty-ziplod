



use serenity::{
    builder::CreateEmbed,
    http::Http,
    model::{
        prelude::{ChannelId, Message, UserId},
        user::User,
    },
    prelude::Context,
    utils::Colour,
};

use crate::ZumborInstances;
use crate::errors::Error;

use super::{
    effects::{Effectable},
    encounter::{self, Encounter},
    player::{self, RollResult},
    ui::{ContinueOption, UI},
};

pub async fn start(ctx: &Context, msg: &Message) -> Result<bool, Error> {
    let user: &User = &msg.author;
    let channel_id = msg.channel_id;

    if let Err(err) = add_player_to_instance(ctx, user.id).await {
        nice_message(
            ctx,
            channel_id,
            "You Fail!".to_string(),
            "You already have a running instance of Zumbor you fool!".to_string(),
        )
        .await
        .unwrap();
        return Err(err);
    };

    let mut player = player::get(ctx, &user.tag(), channel_id).await?;
    let mut ui = UI::builder().context(ctx).channel(channel_id).build();

    loop {
        let mut encounter: Encounter = encounter::fetch(ctx).await?;

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
            player.affect(&effect)
        }

        // Handle lingering effects of the result
        if let Some(effect) = &encounter_result.lingering_effect {
            println!("Added lingering effect: {}", effect.name);
            let mut gain_embed: CreateEmbed = effect.into();
            gain_embed.title(format!("Received a {} {}", effect.name, effect.kind));
            gain_embed.colour::<Colour>((&effect.name).into());
            ui.queue_message(gain_embed);
            player.add_effect(effect.clone())
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
                ui.queue_message(end_embed);
            }
        }
        player.apply_effects();
        player.add_score(1);

        if let Err(err) = ui
            .encounter_result(&encounter_result, &player, current_message)
            .await
        {
            println!("Unable to display the encounter result. {}", err);
        }

        if player.health <= 0 {
            player.effects.clear();

            ui.say(format!("Uh oh {} died", &player.name).as_ref())
                .await;

            if let Err(err) = player::delete(ctx, &player).await {
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

            remove_player_from_instance(ctx, user.id).await;
            return Ok(true);
        }

        let resume_playing: ContinueOption = ui.request_continue(&player).await?;

        if let ContinueOption::Continue = resume_playing {
            continue;
        };

        remove_player_from_instance(ctx, user.id).await;

        ui.queue_message(quick_embed(
            "Resting...".to_owned(),
            Some(player.name.clone() + " takes a break"),
        ));

        match player::save(ctx, &player).await {
            Ok(_saved) => ui.queue_message(quick_embed("Save Succesful.".to_string(), None)),
            Err(err) => {
                println!("{}", err);
                ui.queue_message(quick_embed(
                    "Ruh Roh Wraggy...".to_string(),
                    Some(format!(
                        "Something went wrong while saving... Say goodbye to {}",
                        player.name
                    )),
                ))
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
    ctx: impl AsRef<Http>,
    channel_id: ChannelId,
    title: String,
    description: String,
) -> Result<Message, Error> {
    let res = channel_id
        .send_message(ctx, |msg| {
            msg.embed(|emb| emb.title(title).description(description))
        })
        .await?;
    Ok(res)
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
        Err(Error::Plain("The user currently has an active Zumbor instance"))
    } else {
        zumbor.instances.push(user_id);
        println!("{:?}",zumbor.instances);
        Ok(true)
    }
}

async fn remove_player_from_instance(ctx: &Context, user_id: UserId) {
    let mut data = ctx.data.write().await;
    let zumbor = data.get_mut::<ZumborInstances>().unwrap();
    // Ignore if no such element is found
    println!("{:?}",zumbor.instances);
    if zumbor.instances.iter().filter(|x| **x == user_id).collect::<Vec<&UserId>>().len() == 1 {
        zumbor.instances.remove(0);
    }
}
