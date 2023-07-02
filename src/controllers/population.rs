use std::collections::HashMap;
use auraxis::{Faction, Loadout, WorldID, ZoneID};
use sqlx::PgPool;
use tracing::{error, info};

pub type LoadoutBreakdown = HashMap<Loadout, i16>;

pub type FactionBreakdown = HashMap<Faction, HashMap<Faction, LoadoutBreakdown>>;

pub type ZoneBreakdown = HashMap<ZoneID, FactionBreakdown>;

pub type WorldBreakdown = HashMap<WorldID, ZoneBreakdown>;

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
) -> Result<WorldBreakdown, sqlx::Error> {
    // Log all optional parameters using info!()
    info!("worlds: {:?}", worlds);
    info!("zones: {:?}", zones);
    info!("factions: {:?}", factions);
    info!("team_ids: {:?}", team_ids);
    info!("loadouts: {:?}", loadouts);
    let population = sqlx::query!(
        "SELECT wp.timestamp, wp.world_id, zp.zone_id, fp.faction_id, fp.team_id, lp.loadout_id, lp.amount FROM world_population wp
        JOIN zone_population zp ON wp.population_id = zp.world_population_id
        JOIN faction_population fp ON zp.zone_population_id = fp.zone_population_id
        JOIN loadout_population lp ON fp.faction_population_id = lp.faction_population_id
        WHERE ($1::INTEGER[] IS NULL OR wp.world_id = ANY($1::INTEGER[]))
            AND ($2::INTEGER[] IS NULL OR zp.zone_id = ANY($2::INTEGER[]))
            AND ($3::SMALLINT[] IS NULL OR fp.faction_id = ANY($3::SMALLINT[]))
            AND ($4::SMALLINT[] IS NULL OR fp.team_id = ANY($4::SMALLINT[]))
            AND ($5::SMALLINT[] IS NULL OR lp.loadout_id = ANY($5::SMALLINT[]))
            AND wp.population_id = (
                SELECT MAX(wp2.population_id) FROM world_population wp2 WHERE wp2.world_id = wp.world_id
            )
        ORDER BY wp.timestamp",
        worlds,
        zones,
        factions,
        team_ids,
        loadouts,
    )
    .fetch_all(db_pool)
    .await?;

    let mut world_breakdown: WorldBreakdown = HashMap::new();

    for record in population {
        let world_id = match WorldID::try_from(record.world_id as i16) {
            Ok(world_id) => world_id,
            Err(_) => {
                error!("Invalid world ID is not defined in auraxis-rs: {}", record.world_id);
                continue;
            },
        };
        let zone_id = record.zone_id as ZoneID;
        let faction_id = match Faction::try_from(record.faction_id) {
            Ok(faction_id) => faction_id,
            Err(_) => {
                error!("Invalid faction ID is not defined in auraxis-rs: {}", record.faction_id);
                continue;
            },
        };
        let team_id = match Faction::try_from(record.team_id) {
            Ok(team_id) => team_id,
            Err(_) => {
                error!("Invalid team ID (Faction enum) is not defined in auraxis-rs: {}", record.team_id);
                continue;
            },
        };
        let loadout_id = match Loadout::try_from(record.loadout_id) {
            Ok(loadout_id) => loadout_id,
            Err(_) => {
                error!("Invalid loadout ID is not defined in auraxis-rs: {}", record.loadout_id);
                continue;
            },
        };
        let amount = record.amount;

        let world = world_breakdown.entry(world_id).or_insert_with(HashMap::new);
        let zone = world.entry(zone_id).or_insert_with(HashMap::new);
        let faction = zone.entry(faction_id).or_insert_with(HashMap::new);
        let team = faction.entry(team_id).or_insert_with(HashMap::new);
        let loadout = team.entry(loadout_id).or_insert(0);

        *loadout += amount;
    }

    Ok(world_breakdown)
}
