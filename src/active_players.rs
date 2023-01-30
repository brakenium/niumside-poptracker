use sqlx::Pool;
use sqlx::Postgres;
use tracing::info;
use chrono;
use tracing::warn;
use std::sync::PoisonError;
use std::time::Duration;
use tokio;
use auraxis::CharacterID;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use chrono::Utc;
use chrono::DateTime;
use auraxis::{WorldID, Loadout, ZoneID};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct ActivePlayer {
    pub world: WorldID,
    pub zone: ZoneID,
    pub loadout: Loadout,
    pub last_change: DateTime<Utc>,
}

type ActivePlayerHashmap = HashMap<CharacterID, ActivePlayer>;

pub type ActivePlayerDb = Arc<Mutex<ActivePlayerHashmap>>;

pub type LoadoutBreakdown = HashMap<WorldID, HashMap<ZoneID, HashMap<Loadout, u16>>>;

pub async fn clean_active_players(active_players: ActivePlayerDb) -> Option<()> {
    let active_players = active_players.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let mut active_players_lock = active_players.lock().unwrap();
        active_players_lock.retain(|_character_id, player| {
            player.last_change + chrono::Duration::minutes(3) > Utc::now()
        });
        info!("Cleaned active players");
    }
}

pub fn loadout_breakdown(active_players: &ActivePlayerDb) -> LoadoutBreakdown {
    let mut loadout_breakdown: LoadoutBreakdown = HashMap::new();
    let active_players_lock = active_players.lock().unwrap();

    for (_, player) in active_players_lock.iter() {
        loadout_breakdown
            .entry(player.world)
            .or_insert_with(HashMap::new)
            .entry(player.zone)
            .or_insert_with(HashMap::new)
            .entry(player.loadout)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    };

    loadout_breakdown
}

// Create a function to generate the SQL queries to store all unique datatypes in the database.
pub async fn generate_unique_datatype_queries(loadout_breakdown: &LoadoutBreakdown) {
    let mut world_ids: Vec<WorldID> = Vec::new();
    let mut zone_ids: Vec<ZoneID> = Vec::new();
    let mut loadout_ids: Vec<Loadout> = Vec::new();

    for (world_id, zone_map) in loadout_breakdown.iter() {
        world_ids.push(*world_id);
        for (zone_id, loadout_map) in zone_map.iter() {
            zone_ids.push(*zone_id);
            for (loadout_id, _count) in loadout_map.iter() {
                loadout_ids.push(*loadout_id);
            }
        }
    }

    world_ids.dedup();
    zone_ids.sort();
    zone_ids.dedup();
    loadout_ids.dedup();

    world_ids.par_iter().count();

    info!("World IDs: {:?}", world_ids.par_iter().count());
    info!("Zone IDs: {:?}", zone_ids.par_iter().count());
    info!("Loadout IDs: {:?}", loadout_ids.par_iter().count());
}


pub async fn store_pop(loadout_breakdown: &LoadoutBreakdown, db_pool: &Pool<Postgres>) {
}

pub async fn process_loop(active_players: ActivePlayerDb, db_pool: Pool<Postgres>) -> Option<()> {
    let active_players = active_players.clone();
    let db_pool = db_pool.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let loadout_breakdown_numbers = loadout_breakdown(&active_players);
        generate_unique_datatype_queries(&loadout_breakdown_numbers).await;
        store_pop(&loadout_breakdown_numbers, &db_pool).await;
    }
}
 