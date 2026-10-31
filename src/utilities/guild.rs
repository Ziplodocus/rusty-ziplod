use serenity::model::prelude::{ChannelType, GuildChannel, Member};

use crate::{Context, errors::Error};

pub async fn get_voice_channel(ctx: Context<'_>, member: &Member) -> Result<GuildChannel, Error> {
    let guild = ctx
        .guild_id()
        .ok_or(Error::Plain("Can only be used in a server"))?;
    let channels = guild.channels(ctx).await.expect("Guild is available");

    for (_, channel) in channels {
        if channel.kind != ChannelType::Voice {
            continue;
        };

        let members = channel
            .members(ctx)
            .expect("A voice channel has the concept of members");

        if members
            .iter()
            .any(move |channel_member| channel_member.user == member.user)
        {
            return Ok(channel);
        }
    }

    Err(Error::Plain("No voice channel found for that user"))
}
