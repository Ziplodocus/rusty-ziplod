
use std::{sync::Arc, time::Duration};

use serenity::{
    model::prelude::{
        interaction::{
            message_component::MessageComponentInteraction, modal::ModalSubmitInteraction,
        },
        Message,
    },
    prelude::Context,
};

use crate::errors::Error;

pub(crate) async fn component(
    message: &Message,
    context: &Context,
    user_tag: Arc<str>,
) -> Result<Arc<MessageComponentInteraction>, Error> {
    message
        .await_component_interaction(context)
        .filter(move |interaction| interaction.user.tag() == user_tag.as_ref())
        .timeout(Duration::new(240, 0))
        .collect_limit(1)
        .await
        .ok_or(Error::Plain(
            "Message Component interaction was not collected",
        ))
}
pub(crate) async fn modal(
    message: &Message,
    context: &Context,
    user_tag: Arc<str>,
) -> Result<Arc<ModalSubmitInteraction>, Error> {
    message
        .await_modal_interaction(context)
        .filter(move |interaction| interaction.user.tag() == user_tag.as_ref())
        .timeout(Duration::new(120, 0))
        .collect_limit(1)
        .await
        .ok_or(Error::Plain("Modal interaction was not collected"))
}
