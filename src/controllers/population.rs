use crate::census::constants::{Loadout, TeamID, WorldID, ZoneID};
use crate::serde::naivedatetime;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::error;
use utoipa::ToSchema;

pub type PopulationAmount = u16;

pub type LoadoutBreakdown = HashMap<Loadout, PopulationAmount>;

pub type TeamBreakdown = HashMap<TeamID, LoadoutBreakdown>;

pub type ZoneBreakdown = HashMap<ZoneID, TeamBreakdown>;

pub type WorldBreakdown = HashMap<WorldID, ZoneBreakdown>;

pub struct PopBreakdown {
    pub timestamp: chrono::NaiveDateTime,
    pub worlds: WorldBreakdown,
}

#[derive(Serialize, ToSchema)]
pub struct PopulationApiResponse {
    #[serde(with = "naivedatetime")]
    pub timestamp: chrono::NaiveDateTime,
    pub worlds: Vec<PopWorld>,
}

#[derive(Serialize, ToSchema)]
pub struct PopWorld {
    pub world_id: WorldID,
    pub world_population: u16,
    pub zones: Vec<PopZone>,
}

#[derive(Serialize, ToSchema)]
pub struct PopZone {
    pub zone_id: ZoneID,
    pub zone_population: u16,
    pub teams: Vec<PopTeam>,
}

#[derive(Serialize, ToSchema)]
pub struct PopTeam {
    pub team_id: TeamID,
    pub team_population: u16,
    pub loadouts: Vec<PopLoadout>,
}

#[derive(Serialize, ToSchema)]
pub struct PopLoadout {
    pub loadout_id: Loadout,
    pub loadout_population: u16,
}

// Get the current population from the database as a tree
//
// # Arguments
//
// * `db_pool` - The database pool to use
// * `worlds` - The world IDs to check
// * `zones` - The zone IDs to check
// * `teams` - The team IDs to check
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
    teams: Option<&[i16]>,
    loadouts: Option<&[i16]>,
) -> Option<PopBreakdown> {
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
        teams,
        loadouts,
    )
        .fetch_all(db_pool)
        .await else {
            return None;
        };

    let mut world_breakdown: WorldBreakdown = HashMap::new();

    let timestamp = population[0].timestamp;

    for record in population {
        #[allow(clippy::cast_possible_truncation)]
        let Ok(world_id) = WorldID::try_from(record.world_id as u16) else {
            error!(
                "Invalid world ID is not defined in auraxis-rs: {}",
                record.world_id
            );
            continue;
        };
        let Ok(team_id) = TeamID::try_from(record.team_id as u16) else {
            error!(
                "Invalid team ID (Faction enum) is not defined in auraxis-rs: {}",
                record.team_id
            );
            continue;
        };
        let Ok(loadout_id) = Loadout::try_from(record.loadout_id as u16) else {
            error!(
                "Invalid loadout ID is not defined in auraxis-rs: {}",
                record.loadout_id
            );
            continue;
        };

        let zone_id = ZoneID(record.zone_id as u32);

        let amount = record.amount;

        let world = world_breakdown
            .entry(world_id)
            .or_default();
        let zone = world.entry(zone_id).or_default();
        let team = zone.entry(team_id).or_default();
        let loadout = team.entry(loadout_id).or_insert(0);

        *loadout += amount as u16;
    }

    Some(PopBreakdown {
        timestamp,
        worlds: world_breakdown,
    })
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
pub fn get_pop_worlds_from_world_breakdown(population: PopBreakdown) -> PopulationApiResponse {
    let mut result = Vec::new();
    for (world_id, world_population) in population.worlds {
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
            zones,
        });
    }
    PopulationApiResponse {
        timestamp: population.timestamp,
        worlds: result,
    }
}

/// Get the population from the database as a tree using `get_current`
///
/// # Arguments
///
/// * `db_pool` - The database pool to use
/// * `worlds` - The world IDs to check
/// * `zones` - The zone IDs to check
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
    teams: Option<&[i16]>,
    loadouts: Option<&[i16]>,
) -> Option<PopulationApiResponse> {
    let population = get_current(db_pool, worlds, zones, teams, loadouts).await?;

    let result = get_pop_worlds_from_world_breakdown(population);

    Some(result)
}
