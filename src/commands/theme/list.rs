use crate::{CommandResult, Context};

#[poise::command(slash_command, prefix_command)]
pub async fn list(ctx: Context<'_>) -> CommandResult {
    let _ = ctx.defer_ephemeral().await;

    let storage_client = &(ctx.data().storage_client);
    let user = ctx.author();
    let tag = user.tag();

    let intro_path = format!("themes/{}/intro", tag);
    let outro_path = format!("themes/{}/outro", tag);

    let intros = storage_client.get_objects(&intro_path);
    let outros = storage_client.get_objects(&outro_path);

    let (intros, outros) = tokio::join!(intros, outros);

    let mut reply: String = format!("{tag}:\nIntros:");

    let intro_list: Vec<String> = intros
        .unwrap_or(Vec::new())
        .into_iter()
        .map(|theme| theme.name.replace(&intro_path, ""))
        .collect();
    let outro_list: Vec<String> = outros
        .unwrap_or(Vec::new())
        .into_iter()
        .map(|theme| theme.name.replace(&outro_path, ""))
        .collect();

    reply += "\n\t";
    reply += intro_list.join("\n\t").as_str();
    reply += "\nOutros:\n\t";
    reply += outro_list.join("\n\t").as_str();

    let _ = ctx.say(reply).await;

    Ok(())
}
