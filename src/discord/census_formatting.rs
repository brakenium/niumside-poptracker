#[cfg(feature = "census")]
use crate::census::constants::Faction;
#[cfg(feature = "census")]
use crate::controllers::population::{PopWorld, PopulationApiResponse};
use crate::controllers::zone::Zone;
#[cfg(feature = "census")]
use crate::discord::icons::Icons;
use crate::utils::safe_percentage;
use chrono::Utc;
use poise::serenity_prelude;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use tracing::error;

struct TotalPopulation {
    pub faction: Faction,
    pub population: u16,
}

pub fn world_breakdown_message(population_breakdown: &mut PopulationApiResponse, full_zone_data: &Option<Vec<Zone>>) -> Vec<CreateEmbed> {
    let mut embeds = Vec::new();

    for world in &mut population_breakdown.worlds {
        embeds.push(single_world_breakdown_embed(
            world,
            full_zone_data,
            population_breakdown.timestamp,
        ));
    }

    embeds
}

fn create_population_string(
    total_population: &Vec<TotalPopulation>,
    world: &PopWorld,
) -> String {
    let mut population_string = String::new();

    for pop_item in total_population {
        let wrapped_icons = Icons::try_from(pop_item.faction)
            .unwrap_or(Icons::Ps2White)
            .to_discord_emoji();

        let icon: String = wrapped_icons.map_or_else(
            || pop_item.faction.to_string(),
            |emoji| emoji.to_string(),
        );

        let percentage = safe_percentage(pop_item.population, world.world_population);

        population_string = format!("{}{}: {} ({:.2})\n", population_string, icon, pop_item.population, percentage);
    }

    format!("{}Total: {}\n", population_string, world.world_population)
}

fn get_total_population(world: &PopWorld) -> Vec<TotalPopulation> {
    let mut total_population: Vec<TotalPopulation> = Vec::new();

    for zone in &world.zones {
        for team in &zone.teams {
            let team_index = total_population
                .iter()
                .position(|p| p.faction == team.team_id);

            match team_index {
                Some(index) => {
                    total_population[index].population += team.team_population;
                }
                None => {
                    total_population.push(TotalPopulation {
                        faction: team.team_id,
                        population: team.team_population,
                    });
                }
            }
        }
    }

    total_population
}

fn create_population_embed_base() -> CreateEmbed {
    CreateEmbed::default()
        .thumbnail("https://www.planetside2.com/images/ps2-logo.png")
        .description("This overview is based on active players earning XP.")
}

fn add_timestamp_to_embed(
    mut embed: CreateEmbed,
    datetime: chrono::DateTime<Utc>,
) -> CreateEmbed {
    match serenity_prelude::Timestamp::from_unix_timestamp(datetime.timestamp()) {
        Ok(timestamp) => {
            embed = embed.timestamp(timestamp);
        }
        Err(e) => {
            error!("Failed to convert timestamp to Discord timestamp, using string in footer: u{:?}", e);

            let footer = CreateEmbedFooter::new(format!("Last updated: {datetime}"));
            embed = embed.footer(footer);
        }
    }

    embed
}

pub fn single_world_breakdown_embed(
    world: &mut PopWorld,
    full_zone_data: &Option<Vec<Zone>>,
    timestamp: chrono::NaiveDateTime,
) -> CreateEmbed {
    let mut total_population = get_total_population(world);

    total_population.sort_by_key(|p| p.faction as u16);

    let global_population_string = create_population_string(&total_population, world);

    let embed = create_population_embed_base()
        .title(format!("{} Population", world.world_id))
        .field("Global Population", global_population_string, false);

    let mut embed = add_timestamp_to_embed(embed, timestamp.and_utc());

    world.zones.sort_by(|a, b| b.zone_population.cmp(&a.zone_population));

    for zone in &world.zones {
        let mut breakdown = String::new();

        let mut sorted_teams = zone.teams.clone();
        sorted_teams.sort_by_key(|t| t.team_id);

        for team in sorted_teams {
            let icon = Icons::try_from(team.team_id)
                .unwrap_or(Icons::Ps2White)
                .to_discord_emoji();

            let team_icon: String = match icon {
                Some(emoji) => emoji.to_string(),
                None => team.team_id.to_string(),
            };

            let percentage = safe_percentage(team.team_population, zone.zone_population);

            breakdown = format!("{}{}: {} ({:.2}%)\n", breakdown, team_icon, team.team_population, percentage);
        }

        breakdown = format!("{}Total: {}\n", breakdown, zone.zone_population);

        #[allow(clippy::cast_sign_loss)]
        let zone_name = full_zone_data.as_ref().map_or_else(|| zone.zone_id.to_string(), |zones| {
            zones
                .iter()
                .find(|z| z.id as u32 == zone.zone_id.0)
                .map_or_else(
                    || zone.zone_id.to_string(),
                    |z| {
                        z.name.as_ref()
                            .map_or_else(|| zone.zone_id.to_string(), |name|
                                name.en.clone().unwrap_or_else(|| zone.zone_id.to_string())
                            )
                    },
                )
        });

        let main_continent = [2, 4, 6, 8, 10, 344].contains(&zone.zone_id.0);

        let percentage = safe_percentage(zone.zone_population, world.world_population);

        embed = embed.field(
            format!("{zone_name} ({percentage:.2}%)"),
            breakdown,
            !main_continent,
        );
    }

    embed
}
