use std::collections::HashMap;
use serde::Serialize;
use sqlx::PgPool;
use tracing::{error, info};
use utoipa::ToSchema;
use crate::census::constants::{Faction, Loadout, WorldID};

pub type LoadoutBreakdown = HashMap<i16, i16>;

pub type FactionBreakdown = HashMap<i16, LoadoutBreakdown>;

pub type ZoneBreakdown = HashMap<i32, FactionBreakdown>;

pub type WorldBreakdown = HashMap<i32, (chrono::NaiveDateTime, ZoneBreakdown)>;

#[derive(Serialize, ToSchema)]
pub struct PopWorld {
    pub world_id: i32,
    pub world_population: i16,
    pub timestamp: chrono::NaiveDateTime,
    pub zones: Vec<PopZone>,
}

#[derive(Serialize, ToSchema)]
pub struct PopZone {
    pub zone_id: i32,
    pub zone_population: i16,
    pub teams: Vec<PopTeam>,
}

#[derive(Serialize, ToSchema)]
pub struct PopTeam {
    pub team_id: i16,
    pub team_population: i16,
    pub loadouts: Vec<PopLoadout>,
}

#[derive(Serialize, ToSchema)]
pub struct PopLoadout {
    pub loadout_id: i16,
    pub loadout_population: i16,
}

// Get the current population from the database as a tree
//
// # Arguments
//
// * `db_pool` - The database pool to use
// * `worlds` - The world IDs to check
// * `zones` - The zone IDs to check
// * `factions` - The faction IDs to check
// * `team_ids` - The team IDs to check
// * `loadouts` - The loadout IDs to check
//
// # Returns
//
// * `Ok(WorldPopulation)` - A hashmap containing the current population
// * `Err(sqlx::Error)` - The error returned by sqlx
pub async fn get_current(
    db_pool: &PgPool,
    worlds: Option<&[i32]>,
    zones: Option<&[i32]>,
    factions: Option<&[i16]>,
    team_ids: Option<&[i16]>,
    loadouts: Option<&[i16]>,
) -> Option<WorldBreakdown> {
    let Ok(population) = sqlx::query!(
        "SELECT
            p.timestamp,
            wp.world_id,
            zp.zone_id,
            tp.team_id,
            lp.loadout_id,
            lp.amount
        FROM population p
        JOIN world_population wp ON p.population_id = wp.population_id
        JOIN zone_population zp ON wp.world_population_id = zp.world_population_id
        JOIN team_population tp ON zp.zone_population_id = tp.zone_population_id
        JOIN loadout_population lp ON tp.team_population_id = lp.team_population_id
        WHERE ($1::INTEGER[] IS NULL OR wp.world_id = ANY($1::INTEGER[]))
            AND ($2::INTEGER[] IS NULL OR zp.zone_id = ANY($2::INTEGER[]))
            AND ($3::SMALLINT[] IS NULL OR tp.team_id = ANY($3::SMALLINT[]))
            AND ($4::SMALLINT[] IS NULL OR lp.loadout_id = ANY($4::SMALLINT[]))
            AND wp.population_id = (
                SELECT MAX(wp2.population_id) FROM world_population wp2 WHERE wp2.world_id = wp.world_id
            )
        ORDER BY p.timestamp",
        worlds,
        zones,
        team_ids,
        loadouts,
    )
        .fetch_all(db_pool)
        .await else {
            return None;
        };

    let mut world_breakdown: WorldBreakdown = HashMap::new();

    for record in population {
        #[allow(clippy::cast_possible_truncation)]
            let Ok(_) = WorldID::try_from(record.world_id as i16) else {
            error!("Invalid world ID is not defined in auraxis-rs: {}", record.world_id);
            continue;
        };
        let Ok(_) = Faction::try_from(record.team_id) else {
            error!("Invalid team ID (Faction enum) is not defined in auraxis-rs: {}", record.team_id);
            continue;
        };
        let Ok(_) = Loadout::try_from(record.loadout_id) else {
            error!("Invalid loadout ID is not defined in auraxis-rs: {}", record.loadout_id);
            continue;
        };

        let world_id = record.world_id;
        let team_id = record.team_id;
        let loadout_id = record.loadout_id;
        let zone_id = record.zone_id;
        let amount = record.amount;

        let world = world_breakdown.entry(world_id).or_insert_with(|| (record.timestamp, HashMap::new()));
        let zone = world.1.entry(zone_id).or_insert_with(HashMap::new);
        let team = zone.entry(team_id).or_insert_with(HashMap::new);
        let loadout = team.entry(loadout_id).or_insert(0);

        *loadout += amount;
    }

    Some(world_breakdown)
}

/// Get `PopWorld` from `WorldBreakdown`
///
/// # Arguments
///
/// * `world_breakdown` - The `WorldBreakdown` to convert
///
/// # Returns
///
/// * `Vec<PopWorld>` - The converted `WorldBreakdown`
pub fn get_pop_worlds_from_world_breakdown(population: WorldBreakdown) -> Vec<PopWorld> {
    let mut result = Vec::new();
    for (world_id, (timestamp, world_population)) in population {
        let mut zones = Vec::new();
        for (zone_id, zone_population) in world_population {
                let mut teams = Vec::new();
                for (team_id, team_population) in zone_population {
                    let mut loadouts = Vec::new();
                    for (loadout_id, loadout_population) in team_population {
                        loadouts.push(PopLoadout {
                            loadout_id,
                            loadout_population,
                        });
                    }
                    teams.push(PopTeam {
                        team_id,
                        team_population: loadouts.iter().map(|l| l.loadout_population).sum(),
                        loadouts,
                    });
                }
            zones.push(PopZone {
                zone_id,
                zone_population: teams.iter().map(|t| t.team_population).sum(),
                teams,
            });
        }
        result.push(PopWorld {
            world_id,
            world_population: zones.iter().map(|z| z.zone_population).sum(),
            timestamp,
            zones,
        });
    }
    result
}

/// Get the population from the database as a tree using get_current
///
/// # Arguments
///
/// * `db_pool` - The database pool to use
/// * `worlds` - The world IDs to check
/// * `zones` - The zone IDs to check
/// * `factions` - The faction IDs to check
/// * `team_ids` - The team IDs to check
/// * `loadouts` - The loadout IDs to check
///
/// # Returns
///
/// * `Ok(PopWorld)` - A hashmap containing the current population
/// * `Err(sqlx::Error)` - The error returned by sqlx
pub async fn get_current_tree(
    db_pool: &PgPool,
    worlds: Option<&[i32]>,
    zones: Option<&[i32]>,
    factions: Option<&[i16]>,
    teams: Option<&[i16]>,
    loadouts: Option<&[i16]>
) -> Option<Vec<PopWorld>> {
    let population = get_current(
        db_pool,
        worlds,
        zones,
        factions,
        teams,
        loadouts,
    ).await?;

    let result = get_pop_worlds_from_world_breakdown(population);

    Some(result)
}
