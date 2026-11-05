use std::pin::Pin;

use serenity::{all::prelude::EventHandler, model::voice::VoiceState};

use poise::{FrameworkContext, serenity_prelude as serenity};

use crate::{CommandError, Context, Data, commands::theme::get_theme_path, errors::Error, voice};

pub async fn event_handler(
    framework: poise::FrameworkContext<'_, Data, CommandError>,
    event: &serenity::FullEvent,
) -> Result<(), CommandError> {
    let data = framework.user_data;
    let ctx = framework.serenity_context;

    match event {
        serenity::FullEvent::VoiceStateUpdate { old, new } => match old {
            None => {
                let Some(channel) = new.channel_id else {
                    return Ok(());
                };
                let Some(member) = &new.member else {
                    return Ok(());
                };
                let Some(guild_id) = new.guild_id else {
                    return Ok(());
                };

                let tag = member.user.tag();
                let storage_client = &data.storage_client;

                let path = get_theme_path(&tag, "intro", None, storage_client)
                    .await
                    .unwrap();
                let file_stream = storage_client.get_stream(&path).await.unwrap();

                let _ = voice::play(ctx, channel, guild_id, file_stream).await;

                return Ok(());
            }
            Some(old) => {
                let None = new.channel_id else {
                    return Ok(());
                };
                let Some(channel) = old.channel_id else {
                    return Ok(());
                };
                let Some(member) = &old.member else {
                    return Ok(());
                };
                let Some(guild_id) = old.guild_id else {
                    return Ok(());
                };

                let tag = member.user.tag();
                let storage_client = &data.storage_client;

                let path = get_theme_path(&tag, "outro", None, storage_client)
                    .await
                    .unwrap();
                let file_stream = storage_client.get_stream(&path).await.unwrap();

                let _ = voice::play(ctx, channel, guild_id, file_stream).await;

                return Ok(());
            }
        },
        _ => {}
    }
    Ok(())
}
