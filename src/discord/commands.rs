use crate::census::constants::WorldID;
use crate::controllers::population;
use crate::discord::{formatting, Context, Error};
use poise::{serenity_prelude, CreateReply};
use strum::IntoEnumIterator;

/// Displays your or another user's account creation date
#[poise::command(slash_command, track_edits)]
pub async fn population(
    ctx: Context<'_>,
    #[description = "The Planetside 2 server to show population for"]
    #[autocomplete = "world_id_autocomplete"]
    server: i32,
) -> Result<(), Error> {
    let Some(population) = population::get_current_tree(
        &ctx.data().db_pool.clone(),
        Some(&[server]),
        None,
        None,
        None,
    )
    .await
    else {
        return Err(Error::from("Failed to get population"));
    };

    let response = formatting::world_breakdown_message(&population);

    let mut reply = CreateReply::default();
    reply.embeds.extend(response);

    ctx.send(reply).await?;

    Ok(())
}

#[allow(clippy::unused_async)]
async fn world_id_autocomplete(
    _ctx: Context<'_>,
    _partial: &str,
) -> impl Iterator<Item = serenity_prelude::AutocompleteChoice> {
    WorldID::iter().map(|v| serenity_prelude::AutocompleteChoice::new(format!("{v}"), v as i16))
}
