use crate::census::constants::WorldID;
use crate::controllers::{population, zone};
use crate::discord::formatters;
use crate::discord::{Context, Error};
use poise::{serenity_prelude, CreateReply};
use strum::IntoEnumIterator;
use tracing::error;

/// Displays your or another user's account creation date
#[poise::command(slash_command, track_edits)]
pub async fn population(
    ctx: Context<'_>,
    #[description = "The Planetside 2 server to show population for"]
    #[autocomplete = "world_id_autocomplete"]
    server: i32,
) -> Result<(), Error> {
    // Defer gives the bot longer to respond, so we don't get a "This interaction failed" error
    ctx.defer().await?;

    let Some(mut population) = population::get_current_tree(
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

    let full_zone_data = match zone::get_all(&ctx.data().db_pool.clone()).await {
        Ok(zones) => Some(zones),
        Err(e) => {
            error!("Failed to get zone data: {:?}", e);
            None
        }
    };

    // Sort population by world ID and then by faction ID
    population.worlds.sort_by_key(|w| w.world_id);
    for world in &mut population.worlds {
        world.zones.sort_by_key(|z| z.zone_id);
        for zone in &mut world.zones {
            zone.teams.sort_by_key(|t| t.team_id);
        }
    }

    let response = formatters::census::world_breakdown_message(&mut population, &full_zone_data);

    let final_reply = CreateReply {
        embeds: response,
        ..CreateReply::default()
    };

    ctx.send(final_reply).await?;

    Ok(())
}

#[allow(clippy::unused_async)]
async fn world_id_autocomplete<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Iterator<Item = serenity_prelude::AutocompleteChoice> + 'a {
    // WorldID::iter().map(|v| serenity_prelude::AutocompleteChoice::new(format!("{v}"), v as i16))
    //     Use partial to search for World that contains the partial string
    WorldID::iter()
        .filter(move |v| {
            v.to_string()
                .to_lowercase()
                .contains(&partial.to_lowercase())
        })
        .map(|v| serenity_prelude::AutocompleteChoice::new(format!("{v}"), v as i16))
}
