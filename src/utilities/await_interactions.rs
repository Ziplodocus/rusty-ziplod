use std::{sync::Arc, time::Duration};

use serenity::{
    all::{ComponentInteraction, ModalInteraction},
    model::prelude::Message,
    prelude::Context,
};

use crate::errors::Error;

pub(crate) async fn component(
    context: &Context,
    message: &Message,
    user_tag: Arc<str>,
) -> Result<ComponentInteraction, Error> {
    message
        .await_component_interaction(context)
        .filter(move |interaction| interaction.user.tag() == user_tag.as_ref())
        .timeout(Duration::new(240, 0))
        .await
        .ok_or(Error::Plain(
            "Message Component interaction was not collected",
        ))
}
pub(crate) async fn modal(
    context: &Context,
    message: &Message,
    user_tag: Arc<str>,
) -> Result<ModalInteraction, Error> {
    message
        .await_modal_interaction(context)
        .filter(move |interaction| interaction.user.tag() == user_tag.as_ref())
        .timeout(Duration::new(120, 0))
        .await
        .ok_or(Error::Plain("Modal interaction was not collected"))
}
