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

pub async fn store_pop(loadout_breakdown: &LoadoutBreakdown, db_pool: &Pool<Postgres>) {
    for (world_id, loadout_breakdown_world) in loadout_breakdown {
        let db_conn = match db_pool.begin().await {
            Ok(conn) => conn,
            Err(error) => {
                tracing::info!("Unable to obtain a connection and start a transaction for world pop insert: {}", error);
                continue;
            },
        };

        let world_query = match sqlx::query!("--sql
            INSERT INTO world_population (world_id, timestamp)
            VALUES ($1, NOW())
            RETURNING population_id",
            *world_id as i16
        ).fetch_optional(&mut db_conn).await {
            Ok(query_result) => query_result,
            Err(_) => {
                tracing::info!("Unable to insert world pop");
                continue;
            },
        };
        for (zone_id, loadout_breakdown_zone) in loadout_breakdown_world{
            let zone_query = match sqlx::query!("--sql
                INSERT INTO zone_population (world_population_id, zone_id)
                VALUES ($1, $2)",
                world_query.column(0),
                *zone_id as i16,

            ).fetch_one(&mut db_conn).await {
                Ok(query_result) => query_result,
                Err(_) => {
                    tracing::info!("Unable to insert zone pop");
                    continue;
                },
            };
            for (loadout, loadout_player_amount) in loadout_breakdown_zone {
                pop_inserts.append("--sql
                ".to_string());
            }
        }
    }
}

pub async fn process_loop(active_players: ActivePlayerDb, db_pool: Pool<Postgres>) -> Option<()> {
    let active_players = active_players.clone();
    let db_pool = db_pool.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let loadout_breakdown_numbers = loadout_breakdown(&active_players);

        store_pop(&loadout_breakdown_numbers, &db_pool).await;
    }
}
