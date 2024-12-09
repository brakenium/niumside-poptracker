#![allow(clippy::cast_lossless)]
use crate::census::constants::{CharacterID, Faction, Loadout, WorldID, ZoneID};
use crate::census::event::GainExperience;
use crate::controllers::population::{
    LoadoutBreakdown, TeamBreakdown, WorldBreakdown, ZoneBreakdown,
};
use chrono::{DateTime, Utc};
use metrics::{counter, gauge};
use sqlx::{Pool, Postgres};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::info;

#[derive(Debug, Clone)]
// TODO: Parse zone ID into zone ID and instance ID
pub struct ActivePlayer {
    pub world: WorldID,
    pub zone: ZoneID,
    pub loadout: Loadout,
    pub team_id: Faction,
    pub last_change: DateTime<Utc>,
}

impl From<GainExperience> for ActivePlayer {
    fn from(event: GainExperience) -> Self {
        Self {
            world: event.world_id,
            zone: event.zone_id,
            loadout: event.loadout_id,
            team_id: event.team_id,
            last_change: event.timestamp,
        }
    }
}

pub type ActivePlayerHashmap = HashMap<CharacterID, ActivePlayer>;

pub type ActivePlayerDb = Arc<Mutex<ActivePlayerHashmap>>;

pub async fn clean(active_players: ActivePlayerDb) -> Option<()> {
    let active_players = active_players.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;

        match active_players.lock() {
            Ok(mut guard) => {
                guard.retain(|_character_id, player| {
                    player.last_change + chrono::Duration::minutes(3) > Utc::now()
                });
            }
            Err(e) => {
                counter!("niumside_active_players_lock_failed").increment(1);
                panic!("Failed to lock active_players: {e}");
            }
        }
        info!("Cleaned active players");
        counter!("niumside_active_players_cleanups").increment(1);
    }
}

pub fn loadout_breakdown(active_players: &ActivePlayerDb) -> WorldBreakdown {
    let mut loadout_breakdown: WorldBreakdown = HashMap::new();
    let active_players_lock = active_players
        .lock()
        .unwrap_or_else(|poisoned| {
            counter!("niumside_active_players_lock_failed").increment(1);
            panic!("Failed to lock active_players: {poisoned}");
        })
        .clone();

    let mut total_players = 0;

    for player in active_players_lock.values() {
        loadout_breakdown
            .entry(player.world)
            .or_default()
            .entry(player.zone)
            .or_default()
            .entry(player.team_id)
            .or_default()
            .entry(player.loadout)
            .and_modify(|v| *v += 1)
            .or_insert(1);

        total_players += 1;
    }

    gauge!("niumside_active_players").set(total_players as f64);

    loadout_breakdown
}

async fn insert_loadout(
    loadout_map: &LoadoutBreakdown,
    faction_population_id: i32,
    db_pool: &Pool<Postgres>,
) {
    for (loadout_id, amount) in loadout_map {
        sqlx::query!(
            "INSERT INTO loadout (loadout_id) VALUES ($1) ON CONFLICT DO NOTHING",
            *loadout_id as i32
        )
        .execute(db_pool)
        .await
        .unwrap_or_else(|error| {
            panic!("Failed database insert: {error}");
        });
        sqlx::query!(
            "INSERT INTO loadout_population (loadout_id, team_population_id, amount) VALUES ($1, $2, $3)",
            *loadout_id as i32,
            faction_population_id,
            *amount as i32
        )
            .execute(db_pool
            )
            .await
            .unwrap_or_else(|error| {
                panic!("Failed database insert: {error}");
            });
    }
}

async fn insert_zone(zone_map: &ZoneBreakdown, world_population_id: i32, db_pool: &Pool<Postgres>) {
    for (zone_id, faction_map) in zone_map {
        sqlx::query!(
            "INSERT INTO zone (zone_id) VALUES ($1) ON CONFLICT DO NOTHING",
            zone_id.0 as i64
        )
        .execute(db_pool)
        .await
        .unwrap_or_else(|error| {
            panic!("Failed database insert: {error}");
        });
        let zone_population_id = sqlx::query!(
            "INSERT INTO zone_population (zone_id, world_population_id) VALUES ($1, $2) RETURNING zone_population_id",
            zone_id.0 as i64,
            world_population_id
        )
            .fetch_one(db_pool)
            .await
            .unwrap_or_else(|error| {
                panic!("Failed database insert: {error}");
            })
            .zone_population_id;

        insert_team(faction_map, zone_population_id, db_pool).await;
    }
}

async fn insert_team(team_map: &TeamBreakdown, zone_population_id: i32, db_pool: &Pool<Postgres>) {
    for (team_id, loadout_map) in team_map {
        sqlx::query!(
            "INSERT INTO faction (faction_id) VALUES ($1) ON CONFLICT DO NOTHING",
            *team_id as i32
        )
        .execute(db_pool)
        .await
        .unwrap_or_else(|error| {
            panic!("Failed database insert: {error}");
        });

        let faction_population_id = sqlx::query!(
                "INSERT INTO team_population (team_id, zone_population_id) VALUES ($1, $2) RETURNING team_population_id",
                *team_id as i32,
                zone_population_id
            )
            .fetch_one(db_pool)
            .await
            .unwrap_or_else(|error| {
                panic!("Failed database insert: {error}");
            })
            .team_population_id;

        insert_loadout(loadout_map, faction_population_id, db_pool).await;
    }
}

pub async fn store_pop(loadout_breakdown: &WorldBreakdown, db_pool: &Pool<Postgres>) {
    let population_id =
        sqlx::query!("INSERT INTO population (timestamp) VALUES (default) RETURNING population_id")
            .fetch_one(db_pool)
            .await
            .unwrap_or_else(|error| {
                panic!("Failed database insert: {error}");
            })
            .population_id;

    for (world_id, zone_map) in loadout_breakdown {
        sqlx::query!(
            "INSERT INTO world (world_id) VALUES ($1) ON CONFLICT DO NOTHING",
            *world_id as i32
        )
        .execute(db_pool)
        .await
        .unwrap_or_else(|error| {
            panic!("Failed database insert: {error}");
        });

        let world_population_id = sqlx::query!(
            "INSERT INTO world_population (world_id, population_id) VALUES ($1, $2) RETURNING world_population_id",
            *world_id as i32,
            population_id
        )
            .fetch_one(db_pool)
            .await
            .unwrap_or_else(|error| {
                panic!("Failed database insert: {error}");
            })
            .world_population_id;

        insert_zone(zone_map, world_population_id, db_pool).await;
    }

    info!("Stored pop");
}

pub async fn process_loop(active_players: ActivePlayerDb, db_pool: Pool<Postgres>) -> Option<()> {
    let active_players = active_players.clone();
    let db_pool = db_pool.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let loadout_breakdown_numbers = loadout_breakdown(&active_players);
        store_pop(&loadout_breakdown_numbers, &db_pool).await;
        counter!("niumside_process_loop_iterations").increment(1);
    }
}
