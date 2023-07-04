use auraxis::WorldID;


use crate::controllers::population;
use crate::discord::{Context, Error, formatting};
use strum::IntoEnumIterator;


/// Displays your or another user's account creation date
#[poise::command(slash_command, track_edits)]
pub async fn population(
    ctx: Context<'_>,
    #[description = "The Planetside 2 server to show population for"]
    #[autocomplete = "world_id_autocomplete"]
    server: i32,
) -> Result<(), Error> {
    let population = population::get_current(
        &ctx.data().db_pool.clone(),
        Some(&[server]),
        None,
        None,
        None,
        None,
    ).await?;

    let mut response = formatting::world_breakdown_message(&population);

    ctx.send(|m| {
        m.embeds.append(&mut response);
        m
    }).await?;

    Ok(())
}

async fn world_id_autocomplete(
    _ctx: Context<'_>,
    _partial: &str,
) -> impl Iterator<Item = poise::AutocompleteChoice<i16>> {
    WorldID::iter()
        .map(|v| poise::AutocompleteChoice {
            name: format!("{v}"),
            value: v as i16,
        })
}
