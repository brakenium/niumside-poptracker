use crate::census::constants::{Loadout, TeamID, WorldID, ZoneID};
use crate::controllers::zone::Zone;
use crate::serde::naivedatetime;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
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

#[derive(Serialize, ToSchema, Clone)]
pub struct PopulationApiResponse {
    #[serde(with = "naivedatetime")]
    pub timestamp: chrono::NaiveDateTime,
    pub worlds: Vec<GenericPopulationLevel>,
}

#[derive(Serialize, ToSchema, Clone)]
#[derive(PartialEq)]
pub enum ValidPopulationLevel {
    Invalid,
    World(Option<WorldID>),
    Zone(Option<ZoneID>),
    Team(Option<TeamID>),
    Loadout(Option<Loadout>),
}

impl ValidPopulationLevel {
    pub fn get_next_level(&self) -> Option<ValidPopulationLevel> {
        match self {
            ValidPopulationLevel::World(_) => Some(ValidPopulationLevel::Zone(None)),
            ValidPopulationLevel::Zone(_) => Some(ValidPopulationLevel::Team(None)),
            ValidPopulationLevel::Team(_) => Some(ValidPopulationLevel::Loadout(None)),
            ValidPopulationLevel::Loadout(_) => None,
            ValidPopulationLevel::Invalid => None,
        }
    }

    pub fn get_db_filter(&self) -> Box<dyn Fn(&&&DbResponse) -> bool> {
        match self {
            ValidPopulationLevel::World(Some(world_id)) => Box::new(move |r| r.world_id == *world_id as i32),
            ValidPopulationLevel::Zone(Some(zone_id)) => Box::new(move |r| r.zone_id == zone_id.0.clone() as i32),
            ValidPopulationLevel::Team(Some(team_id)) => Box::new(move |r| r.team_id == *team_id as i32),
            ValidPopulationLevel::Loadout(Some(loadout_id)) => Box::new(move |r| r.loadout_id == *loadout_id as i32),
            ValidPopulationLevel::Invalid => Box::new(move |_| false),
        }
    }
}

#[derive(Serialize, ToSchema, Clone)]
pub enum ValidFullLevelData {
    Zone(Zone),
}

#[derive(Serialize, ToSchema, Clone)]
pub struct GenericPopulationLevel {
    pub level_id: ValidPopulationLevel,
    pub full_level_data: Option<ValidFullLevelData>,
    pub population: PopulationAmount,
    pub lower_levels: Option<Vec<GenericPopulationLevel>>,
}

struct DbResponse {
    pub timestamp: chrono::NaiveDateTime,
    pub world_id: i32,
    pub zone_id: i32,
    pub team_id: i32,
    pub loadout_id: i32,
    pub amount: i32,
}

pub async fn get_current_from_db(
    db_pool: &PgPool,
    worlds: Option<&[i32]>,
    zones: Option<&[i32]>,
    teams: Option<&[i16]>,
    loadouts: Option<&[i16]>,
) -> Option<Vec<DbResponse>> {
    let Ok(population) = sqlx::query_as!(
        DbResponse,
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

    if population.is_empty() {
        return None;
    }

    Some(population)
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
pub async fn get_current_as_hashmap_tree(
    db_pool: &PgPool,
    worlds: Option<&[i32]>,
    zones: Option<&[i32]>,
    teams: Option<&[i16]>,
    loadouts: Option<&[i16]>,
) -> Option<PopBreakdown> {
    let population = get_current_from_db(db_pool, worlds, zones, teams, loadouts).await?;

    let mut world_breakdown: WorldBreakdown = HashMap::new();

    let timestamp = population[0].timestamp;

    for record in population {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let Ok(world_id) = WorldID::try_from(record.world_id as u16) else {
            error!(
                "Invalid world ID is not defined in auraxis-rs: {}",
                record.world_id
            );
            continue;
        };
        #[allow(clippy::cast_sign_loss)]
        let Ok(team_id) = TeamID::try_from(record.team_id as u16) else {
            error!(
                "Invalid team ID (Faction enum) is not defined in auraxis-rs: {}",
                record.team_id
            );
            continue;
        };
        #[allow(clippy::cast_sign_loss)]
        let Ok(loadout_id) = Loadout::try_from(record.loadout_id as u16) else {
            error!(
                "Invalid loadout ID is not defined in auraxis-rs: {}",
                record.loadout_id
            );
            continue;
        };

        #[allow(clippy::cast_sign_loss)]
        let zone_id = ZoneID(record.zone_id as u32);

        #[allow(clippy::cast_sign_loss)]
        let amount = record.amount as u16;

        let world = world_breakdown
            .entry(world_id)
            .or_default();
        let zone = world.entry(zone_id).or_default();
        let team = zone.entry(team_id).or_default();
        let loadout = team.entry(loadout_id).or_insert(0);

        *loadout += amount;
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
                full_zone_data: None,
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

/// Recursive function to get the population from the database as a tree
/// us used by `get_current_tree`
///
/// # Arguments
///
/// * `records` - The population data to convert
/// * `current_level` - The current level to check
///
fn build_tree(
    records: Vec<&DbResponse>,
    current_level: &ValidPopulationLevel,
) -> Vec<GenericPopulationLevel> {
    // let mut next_levels: Vec<GenericPopulationLevel> = Vec::new();
    //
    // let mut world_ids: Vec<WorldID> = Vec::new();
    //
    // for record in current_records.iter() {
    //     match WorldID::try_from(record.world_id as u16) {
    //         Ok(world_id) => {
    //             world_ids.push(world_id);
    //         }
    //         Err(_) => {
    //             error!(
    //                 "Invalid world ID is not defined: {}",
    //                 record.world_id
    //             );
    //         }
    //     }
    // }
    //
    // for world_id in world_ids {
    //     let world_records: Vec<&DbResponse> = current_records
    //         .iter()
    //         .filter(|r| r.world_id == world_id as i32)
    //         .cloned()
    //         .collect();
    //     let world_population: PopulationAmount = world_records.iter().map(|r| r.amount as u16).sum();
    //     let next_level = build_tree(world_records, ValidPopulationLevel::Zone(None));
    //     next_levels.push(GenericPopulationLevel {
    //         level_id: ValidPopulationLevel::World(Some(world_id)),
    //         full_level_data: None,
    //         population: world_population,
    //         lower_levels: Some(next_level),
    //     });
    // }
    //
    // next_levels
    let mut result = Vec::new();

    let next_level: Option<ValidPopulationLevel> = current_level.get_next_level();

    let mut next_levels: Option<Vec<GenericPopulationLevel>> = None;

    match next_level {
        Some(next_level) => {

            let next_records: Vec<&DbResponse> = records
                .iter()
                .filter(next_level.get_db_filter())
                .cloned()
                .collect();

            let population: PopulationAmount = next_records.iter().map(|r| r.amount as u16).sum();

            next_levels = Some(build_tree(next_records, &next_level));
        },
        None => {},
    }

    // result.push(GenericPopulationLevel {
    //     level_id: current_level,
    //     full_level_data: None,
    //     population: next_levels.iter().map(|l| l.population).sum(),
    //     lower_levels: Some(next_levels),
    // });

    result
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
    let population = get_current_from_db(db_pool, worlds, zones, teams, loadouts).await?;

    let mut world_breakdown: Vec<GenericPopulationLevel> = Vec::new();

    for record in population {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let Ok(world_id) = WorldID::try_from(record.world_id as u16) else {
            error!(
                "Invalid world ID is not defined in auraxis-rs: {}",
                record.world_id
            );
            continue;
        };
        #[allow(clippy::cast_sign_loss)]
        let Ok(team_id) = TeamID::try_from(record.team_id as u16) else {
            error!(
                "Invalid team ID (Faction enum) is not defined in auraxis-rs: {}",
                record.team_id
            );
            continue;
        };
        #[allow(clippy::cast_sign_loss)]
        let Ok(loadout_id) = Loadout::try_from(record.loadout_id as u16) else {
            error!(
                "Invalid loadout ID is not defined in auraxis-rs: {}",
                record.loadout_id
            );
            continue;
        };

        #[allow(clippy::cast_sign_loss)]
        let zone_id = ZoneID(record.zone_id as u32);

        #[allow(clippy::cast_sign_loss)]
        let amount = record.amount as u16;

        let world = world_breakdown
            .iter_mut()
            .find(|w| w.level_id == ValidPopulationLevel::World(world_id))
            .unwrap_or_else(|| {
                let new_world = GenericPopulationLevel {
                    level_id: ValidPopulationLevel::World(world_id),
                    full_level_data: None,
                    population: 0,
                    lower_levels: None,
                };
                world_breakdown.push(new_world);
                world_breakdown.last_mut().unwrap()
            });

        let zone = world
            .lower_levels
            .as_mut()
            .unwrap_or_else(|| {
                world.lower_levels = Some(Vec::new());
                world.lower_levels.as_mut().unwrap()
            })
            .iter_mut()
            .find(|z| z.level_id == ValidPopulationLevel::Zone(zone_id))
            .unwrap_or_else(|| {
                let new_zone = GenericPopulationLevel {
                    level_id: ValidPopulationLevel::Zone(zone_id),
                    full_level_data: None,
                    population: 0,
                    lower_levels: None,
                };
                world.lower_levels.as_mut().unwrap().push(new_zone);
                world.lower_levels.as_mut().unwrap().last_mut().unwrap()
            });
    }

    Ok(PopulationApiResponse {
        timestamp: population[0].timestamp,
        worlds: world_breakdown,
    })
}


