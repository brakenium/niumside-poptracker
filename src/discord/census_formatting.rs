#[cfg(feature = "census")]
use crate::census::constants::Faction;
use crate::controllers::population::PopTeam;
#[cfg(feature = "census")]
use crate::controllers::population::{PopWorld, PopulationApiResponse};
use crate::controllers::zone::Zone;
#[cfg(feature = "census")]
use crate::discord::icons::Icons;
use chrono::Utc;
use poise::serenity_prelude;
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
    let mut total_population: HashMap<Faction, u16> = HashMap::new();

    for zone in &world.zones {
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

            *total_population.entry(team_faction).or_insert(0) += team.team_population;
        }
    }

    let mut description = "This overview is based on active players earning XP.\n\n".to_string();

    for (faction, population) in total_population {
        let icon: String = match Icons::try_from(faction)
            .unwrap_or(Icons::Ps2White)
            .to_discord_emoji() {
            Some(emoji) => emoji.to_string(),
            None => faction.to_string(),
        };

        let percentage = if world.world_population == 0 {
            0.0
        } else {
            (population as f64 / world.world_population as f64) * 100.0
        };

        description = format!("{}{}: {} ({:.2})\n", description, icon, population, percentage);
    }

    description = format!("{}Total: {}\n", description, world.world_population);

    let mut embed = CreateEmbed::default()
        .title(format!("{} Population", world.world_id))
        .thumbnail("https://www.planetside2.com/images/ps2-logo.png".to_string())
        .description(description);

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
