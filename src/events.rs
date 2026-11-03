use std::pin::Pin;

use serenity::{
    all::prelude::{Context, EventHandler},
    model::voice::VoiceState,
};

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    // fn voice_state_update<'a>(
    //     &'a self,
    //     ctx: Context,
    //     old: Option<VoiceState>,
    //     new: VoiceState,
    // ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    //     ()
    // }
}
