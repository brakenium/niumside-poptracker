#[cfg(feature = "census")]
use crate::census::constants::Faction;
#[cfg(feature = "census")]
use crate::controllers::population::{PopWorld, PopulationApiResponse};
use crate::controllers::zone::Zone;
#[cfg(feature = "census")]
use crate::discord::icons::Icons;
use poise::serenity_prelude;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use std::collections::HashMap;
use tracing::error;

struct TotalPopulation {
    pub faction: Faction,
    pub population: u16,
}

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
    let mut total_population: Vec<TotalPopulation> = Vec::new();

    for zone in &world.zones {
        for team in &zone.teams {
            let team_faction = match Faction::try_from(team.team_id) {
                Ok(faction) => faction,
                Err(_) => {
                    error!("Unknown faction ID: {}", team.team_id);
                    Faction::Unknown
                }
            };

            let team_index = total_population
                .iter()
                .position(|p| p.faction == team_faction);

            match team_index {
                Some(index) => {
                    total_population[index].population += team.team_population;
                }
                None => {
                    total_population.push(TotalPopulation {
                        faction: team_faction,
                        population: team.team_population,
                    });
                }
            }
        }
    }

    total_population.sort_by_key(|p| p.faction as u16);

    let mut global_population_string = "".to_string();

    for pop_item in total_population {
        let icon: String = match Icons::try_from(pop_item.faction)
            .unwrap_or(Icons::Ps2White)
            .to_discord_emoji() {
            Some(emoji) => emoji.to_string(),
            None => pop_item.faction.to_string(),
        };

        let percentage = if world.world_population == 0 {
            0.0
        } else {
            (pop_item.population as f64 / world.world_population as f64) * 100.0
        };

        global_population_string = format!("{}{}: {} ({:.2})\n", global_population_string, icon, pop_item.population, percentage);
    }

    global_population_string = format!("{}Total: {}\n", global_population_string, world.world_population);

    let mut embed = CreateEmbed::default()
        .title(format!("{} Population", world.world_id))
        .thumbnail("https://www.planetside2.com/images/ps2-logo.png")
        .description("This overview is based on active players earning XP.")
        .field("Global Population", global_population_string, false);

    match serenity_prelude::Timestamp::from_unix_timestamp(timestamp.and_utc().timestamp()) {
        Ok(timestamp) => {
            embed = embed.timestamp(timestamp);
        }
        Err(e) => {
            error!("Failed to convert timestamp to Discord timestamp, using string in footer: u{:?}", e);

            let footer = CreateEmbedFooter::new(format!("Last updated: {timestamp}"));
            embed = embed.footer(footer);
        }
    }

    let mut sorted_zones = world.zones.clone();
    sorted_zones.sort_by(|a, b| b.zone_population.cmp(&a.zone_population));

    for zone in sorted_zones {
        let mut breakdown = "".to_string();

        let mut sorted_teams = zone.teams.clone();
        sorted_teams.sort_by_key(|t| t.team_id);

        for team in sorted_teams {
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

        breakdown = format!("{}Total: {}\n", breakdown, zone.zone_population);

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

        let main_continent = [2, 4, 6, 8, 10, 344].contains(&zone.zone_id.0);

        let percentage = if world.world_population == 0 {
            0.0
        } else {
            (zone.zone_population as f64 / world.world_population as f64) * 100.0
        };

        embed = embed.field(
            format!("{zone_name} ({:.2}%)", percentage),
            breakdown,
            !main_continent,
        );
    }

    embed
}
