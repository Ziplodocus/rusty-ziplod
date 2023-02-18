use serenity::{
    model::{
        prelude::{ChannelCategory, ChannelId, ChannelType, Message},
        user::User,
    },
    prelude::Context,
    Error,
};

use super::{
    encounter::{Encounter, EncounterResult},
    player::Player,
};

pub struct Display<'a> {
    context: &'a Context,
    user: &'a User,
    channel: ChannelId,
    player: &'a Player,
}

pub struct DisplayBuilder<'a> {
    context: &'a Context,
    user: &'a User,
    channel: ChannelId,
    player: Option<&'a Player>,
}

impl<'a> DisplayBuilder<'a> {
    pub async fn new<'b: 'a>(
        ctx: &'b Context,
        msg: &'b Message,
    ) -> Result<DisplayBuilder<'a>, Error> {
        let channel = msg.channel(ctx).await?;

        let channel_cat = channel.guild().unwrap();

        let channel_type = channel_cat.kind;

        if channel_type != ChannelType::Text {
            return Err(Error::Other("Channel is not of type text channel"));
        }

        let user = &msg.author;
        let channel_id = msg.channel_id;

        Ok(DisplayBuilder {
            context: ctx,
            user,
            channel: channel_id,
            player: None,
        })
    }

    pub fn say(&self, message_content: &str) -> () {
        self.channel.say(self.context, message_content);
    }

    pub async fn request_player(&self) -> Result<Player, Error> {
        let player: Player = serde_json::from_str(
            "
        {
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

    pub fn assign_player<'b: 'a>(&mut self, player: &'b Player) {
        self.player = Some(player);
    }

    pub fn build(&self) -> Result<Display, Error> {
        if let Some(player) = self.player {
            Ok(Display {
                player,
                context: self.context,
                user: self.user,
                channel: self.channel,
            })
        } else {
            Err(Error::Other("No player has been added!"))
        }
    }
}

impl Display<'_> {
    pub async fn say(&self, message_content: &str) -> () {
        self.channel.say(self.context, message_content).await;
    }

    pub async fn encounter_details(&self, _encounter: &Encounter) -> Result<Encounter, Error> {
        todo!()
    }

    pub async fn encounter_options(&self, _encounter: &Encounter) -> Result<&str, Error> {
        todo!()
    }

    pub async fn encounter_result(&self, _encounter: &EncounterResult) -> Result<&str, Error> {
        todo!()
    }

    pub async fn request_continue(&self) -> Result<bool, Error> {
        todo!()
    }
}
