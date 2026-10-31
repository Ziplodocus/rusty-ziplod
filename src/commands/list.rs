use crate::{CommandResult, Context};

use crate::errors::Error;

use cloud_storage::Object;

#[poise::command(slash_command, prefix_command)]
pub async fn list(
    ctx: Context<'_>,
    #[description = "The type of track to list"] track_type: String,
) -> CommandResult {
    let _ = ctx.defer().await;

    let all_tracks = get_tracks(ctx, &track_type).await?;

    ctx.reply(
        all_tracks
            .into_iter()
            .map(|obj| obj.name)
            .collect::<Vec<String>>()
            .join("\n"),
    )
    .await?;

    // println!("Play command ended");
    Ok(())
}

pub async fn get_tracks(ctx: Context<'_>, track_type: &str) -> Result<Vec<Object>, Error> {
    let storage_client = &(ctx.data().storage_client);
    let file_name = format!("tracks/{track_type}/");
    storage_client.get_objects(&file_name).await
}
