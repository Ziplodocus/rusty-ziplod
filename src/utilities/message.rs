use serenity::{
    builder::CreateEmbed,
    model::prelude::{ChannelType, Guild, GuildChannel, Member, Message},
    prelude::Context,
};

use crate::errors::Error;

pub async fn resolve_voice_channel(ctx: &Context, msg: &Message) -> Result<GuildChannel, Error> {
    let user = msg.mentions.first().unwrap_or(&msg.author);
    let guild = msg
        .guild(ctx)
        .expect("Command to be called in a guild channel");

    let maybe_member = guild.member(ctx, user).await;

    let member = maybe_member
        .map_err(|_| Error::Plain("You can't mention someone not in the guild you fool."))?;

    fetch_voice_channel(ctx, &guild, &member).await
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

    Err(Error::Plain("No voice channel found for that user"))
}

pub fn quick_embed(title: String, description: Option<String>) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title(title);
    if let Some(description) = description {
        embed.description(description);
    }

    embed
}
