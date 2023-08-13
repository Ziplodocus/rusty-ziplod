use serenity::{
    model::prelude::{Channel, ChannelType, GuildChannel, Message},
    prelude::Context,
};

pub async fn resolve_voice_channel(ctx: &Context, msg: &Message) -> Option<GuildChannel> {
    let user = msg.mentions.get(0).unwrap_or(&msg.author);
    let guild = msg
        .guild(ctx)
        .expect("Command to be called in a guild channel");

    let maybe_member = guild.member(ctx, user).await;

    if maybe_member.is_err() {
        msg.reply(
            ctx,
            "You can't mention someone not in the channel you fool.",
        );
        return None;
    }
    let member = maybe_member.unwrap();

    let maybe_channel = member.default_channel(ctx);

    if maybe_channel.is_none() {
        msg.reply(
            ctx,
            "The guild member must be in a voice channel you numpty",
        );
        return None;
    }

    let channel = maybe_channel.unwrap();

    if channel.kind != ChannelType::Voice {
        msg.reply(ctx, "Not a voice channel m9");
        return None;
    }

    return Some(channel);
}
