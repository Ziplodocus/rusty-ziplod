use serenity::{
    model::prelude::{ChannelType, Guild, GuildChannel, Member, Message},
    prelude::Context,
};

use crate::errors::Error;

pub async fn resolve_voice_channel(ctx: &Context, msg: &Message) -> Result<GuildChannel, Error> {
    let user = msg.mentions.get(0).unwrap_or(&msg.author);
    let guild = msg
        .guild(ctx)
        .expect("Command to be called in a guild channel");

    let maybe_member = guild.member(ctx, user).await;

    let member = maybe_member.map_err(|_| {
        return Error::Plain("You can't mention someone not in the guild you fool.".into());
    })?;

    return fetch_voice_channel(ctx, &guild, &member).await;
}

pub async fn fetch_voice_channel(
    ctx: &Context,
    guild: &Guild,
    member: &Member,
) -> Result<GuildChannel, Error> {
    let channels = guild.channels(ctx).await.expect("Guild is available");

    for (_, channel) in channels {
        if channel.kind != ChannelType::Voice {
            continue;
        };

        let members = channel
            .members(ctx)
            .await
            .expect("A voice channel has the concept of members");

        if members
            .iter()
            .any(move |channel_member| channel_member.user == member.user)
        {
            return Ok(channel);
        }
    }

    return Err(Error::Plain("No voice channel found for that user"));
}
