use auraxis::{Faction, CharacterID, Loadout, WorldID, ZoneID};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct ActivePlayer {
    pub world: WorldID,
    pub zone: ZoneID,
    pub loadout: Loadout,
    pub faction: Faction,
    pub last_change: DateTime<Utc>,
}

type ActivePlayerHashmap = HashMap<CharacterID, ActivePlayer>;

pub type ActivePlayerDb = Arc<Mutex<ActivePlayerHashmap>>;

pub type LoadoutBreakdown =
    HashMap<WorldID, HashMap<ZoneID, HashMap<Faction, HashMap<Loadout, u16>>>>;

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
        debug!("Active player event: {:?}", player);
        loadout_breakdown
            .entry(player.world)
            .or_insert_with(HashMap::new)
            .entry(player.zone)
            .or_insert_with(HashMap::new)
            .entry(player.faction)
            .or_insert_with(HashMap::new)
            .entry(player.loadout)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    loadout_breakdown
}

struct PopulationID {
    population_id: i32,
}

// Create a function to generate the SQL queries to store all unique datatypes in the database.
pub async fn store_pop(loadout_breakdown: &LoadoutBreakdown, db_pool: &Pool<Postgres>) {
    let mut world_ids: Vec<WorldID> = Vec::new();
    let mut zone_ids: Vec<ZoneID> = Vec::new();
    let mut loadout_ids: Vec<Loadout> = Vec::new();

    for (world_id, zone_map) in loadout_breakdown.iter() {
        world_ids.push(*world_id);
        sqlx::query!(
            "INSERT INTO world (world_id) VALUES ($1) ON CONFLICT DO NOTHING",
            *world_id as i16
        )
        .fetch_one(db_pool)
        .await
        .unwrap();
        let world_population_id = sqlx::query_as!(
            PopulationID,
            "INSERT INTO world_population (world_id) VALUES ($1) RETURNING population_id",
            *world_id as i16
        )
        .fetch_one(db_pool)
        .await
        .unwrap()
        .population_id;

        for (zone_id, faction_map) in zone_map.iter() {
            zone_ids.push(*zone_id);
            sqlx::query!("INSERT INTO zone (zone_id) VALUES ($1)", *zone_id as i16)
                .fetch_one(db_pool)
                .await
                .unwrap();
            let zone_population_id = sqlx::query_as!(
                PopulationID,
                "INSERT INTO zone_population (zone_id, world_population_id) VALUES ($1, $2) RETURNING zone_population_id as population_id",
                *zone_id as i16,
                world_population_id
            ).fetch_one(db_pool).await.unwrap().population_id;

            for (faction_id, loadout_map) in faction_map.iter() {
                sqlx::query!(
                    "INSERT INTO faction (faction_id) VALUES ($1)",
                    *faction_id as i16
                )
                .fetch_one(db_pool)
                .await
                .unwrap();
                let faction_population_id = sqlx::query_as!(
                    PopulationID,
                    "INSERT INTO faction_population (faction_id, zone_population_id) VALUES ($1, $2) RETURNING faction_population_id as population_id",
                    *faction_id as i16,
                    zone_population_id
                ).fetch_one(db_pool).await.unwrap().population_id;

                for (loadout_id, _count) in loadout_map.iter() {
                    loadout_ids.push(*loadout_id);
                    sqlx::query!(
                        "INSERT INTO loadout (loadout_id) VALUES ($1)",
                        *loadout_id as i16
                    )
                    .fetch_one(db_pool)
                    .await
                    .unwrap();
                    sqlx::query!(
                        "INSERT INTO loadout_population (loadout_id, faction_population_id) VALUES ($1, $2)",
                        *loadout_id as i16,
                        faction_population_id
                    ).fetch_one(db_pool).await.unwrap();
                }
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

pub async fn process_loop(active_players: ActivePlayerDb, db_pool: Pool<Postgres>) -> Option<()> {
    let active_players = active_players.clone();
    let db_pool = db_pool.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let loadout_breakdown_numbers = loadout_breakdown(&active_players);
        store_pop(&loadout_breakdown_numbers, &db_pool).await;
        // store_pop(&loadout_breakdown_numbers, &db_pool).await;
    }
}
