#[cfg(feature = "census")]
use crate::census::constants::Faction;
use crate::controllers::population::PopTeam;
#[cfg(feature = "census")]
use crate::controllers::population::{PopWorld, PopulationApiResponse};
use crate::controllers::zone::Zone;
#[cfg(feature = "census")]
use crate::discord::icons::Icons;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;
use std::ops::Sub;
use tracing::error;

pub fn world_breakdown_message(population_breakdown: &PopulationApiResponse, full_zone_data: &Option<Vec<Zone>>) -> Vec<CreateEmbed> {
    let mut embeds = Vec::new();

    for world in &population_breakdown.worlds {
        embeds.push(single_world_breakdown_embed(
            world,
            full_zone_data,
            population_breakdown.timestamp,
        ));
    }

    embeds
}

pub fn single_world_breakdown_embed(
    world: &PopWorld,
    full_zone_data: &Option<Vec<Zone>>,
    timestamp: chrono::NaiveDateTime,
) -> CreateEmbed {
    let footer = CreateEmbedFooter::new(format!("Last updated: {timestamp}"));

    let mut total_population: HashMap<Faction, u16> = HashMap::new();

    for zone in &world.zones {
        for team in &zone.teams {
            let team_faction = match Faction::try_from(team.team_id) {
                Ok(faction) => faction,
                Err(_) => {
                    error!("Unknown faction ID: {}", team.team_id);
                    Faction::Unknown
                }
            };

            *total_population.entry(team_faction).or_insert(0) += team.team_population;
        }
    }

    let mut total_population_string = "".to_string();

    for (faction, population) in total_population {
        let icon: String = match Icons::try_from(faction)
            .unwrap_or(Icons::Ps2White)
            .to_discord_emoji() {
            Some(emoji) => emoji.to_string(),
            None => faction.to_string(),
        };

        total_population_string = format!("{}{}: {}\n", total_population_string, icon, population);
    }

    let mut embed = CreateEmbed::default()
        .title(format!("{} Population", world.world_id))
        .thumbnail("https://www.planetside2.com/images/ps2-logo.png".to_string())
        .footer(footer)
        .field(
            "Server Population",
            total_population_string,
            false,
        );

    for zone in &world.zones {
        let percentage = if world.world_population == 0 {
            0.0
        } else {
            (zone.zone_population as f64 / world.world_population as f64) * 100.0
        };

        let mut breakdown = format!("{} ({:.2}%) active players earning XP\n", zone.zone_population, percentage);

        for team in &zone.teams {
            let team_faction = match Faction::try_from(team.team_id) {
                Ok(faction) => faction,
                Err(_) => {
                    error!("Unknown faction ID: {}", team.team_id);
                    Faction::Unknown
                }
            };

            let team_icon: String = match Icons::try_from(team_faction)
                .unwrap_or(Icons::Ps2White)
                .to_discord_emoji() {
                Some(emoji) => emoji.to_string(),
                None => team_faction.to_string(),
            };

            let percentage = if zone.zone_population == 0 {
                0.0
            } else {
                (team.team_population as f64 / zone.zone_population as f64) * 100.0
            };

            breakdown = format!("{}{}: {} ({:.2}%)\n", breakdown, team_icon, team.team_population, percentage);
        }

        let zone_name = match full_zone_data {
            Some(zones) => zones
                .iter()
                .find(|z| z.zone_id as u32 == zone.zone_id.0)
                .map(|z| {
                    match z.name {
                        Some(ref name) => name.en.clone().unwrap_or(zone.zone_id.to_string()),
                        None => zone.zone_id.to_string(),
                    }
                })
                .unwrap_or(zone.zone_id.to_string()),
            None => zone.zone_id.to_string(),
        };

        embed = embed.field(
            zone_name,
            breakdown,
            false,
        );
    }

    embed
}
