use crate::census::constants::Faction;
use crate::controllers::population::{Pop, PopWorld, WorldBreakdown};
use crate::discord::icons::Icons;
use chrono::Utc;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};

pub fn world_breakdown_message(population_breakdown: &Pop) -> Vec<CreateEmbed> {
    let mut embeds = Vec::new();

    for world in &population_breakdown.worlds {
        embeds.push(single_world_breakdown_embed(
            world,
            population_breakdown.timestamp,
        ));
    }

    embeds
}

pub fn single_world_breakdown_embed(
    world: &PopWorld,
    timestamp: chrono::NaiveDateTime,
) -> CreateEmbed {
    let footer = CreateEmbedFooter::new(format!("Last updated: {timestamp}"));

    let embed = CreateEmbed::default()
        .title(format!("{} Population", world.world_id))
        .thumbnail("https://www.planetside2.com/images/ps2-logo.png".to_string())
        .footer(footer);

    for zone in &world.zones {
        let mut breakdown = String::new();
        for team in &zone.teams {
            let team_icon =
                Icons::try_from(Faction::try_from(team.team_id).unwrap_or(Faction::Unknown))
                    .unwrap_or(Icons::Ps2White)
                    .to_discord_emoji();
            breakdown = format!("{}{:?}: {}\n", breakdown, team_icon, team.team_population);
        }
    }

    embed
}
