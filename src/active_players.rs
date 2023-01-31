use auraxis::{Faction, CharacterID, Loadout, WorldID, ZoneID};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{info};

#[derive(Debug, Clone)]
pub struct ActivePlayer {
    pub world: WorldID,
    pub zone: ZoneID,
    pub loadout: Loadout,
    pub faction: Faction,
    pub team_id: Faction,
    pub last_change: DateTime<Utc>,
}

type ActivePlayerHashmap = HashMap<CharacterID, ActivePlayer>;

pub type ActivePlayerDb = Arc<Mutex<ActivePlayerHashmap>>;

pub type LoadoutBreakdown =
    HashMap<WorldID, HashMap<ZoneID, HashMap<Faction, HashMap<Faction, HashMap<Loadout, u16>>>>>;

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
            .entry(player.faction)
            .or_insert_with(HashMap::new)
            .entry(player.team_id)
            .or_insert_with(HashMap::new)
            .entry(player.loadout)
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    loadout_breakdown
}

pub async fn store_pop(loadout_breakdown: &LoadoutBreakdown, db_pool: &Pool<Postgres>) {
    for (world_id, zone_map) in loadout_breakdown.iter() {
        sqlx::query!(
            "INSERT INTO world (world_id) VALUES ($1) ON CONFLICT DO NOTHING",
            *world_id as i32
        )
        .execute(db_pool)
        .await.unwrap();

        let world_population_id = sqlx::query!(
            "INSERT INTO world_population (world_id) VALUES ($1) RETURNING population_id",
            *world_id as i32
        )
        .fetch_one(db_pool)
        .await
        .unwrap()
        .population_id;

        for (zone_id, faction_map) in zone_map.iter() {
            sqlx::query!(
                "INSERT INTO zone (zone_id) VALUES ($1) ON CONFLICT DO NOTHING",
                *zone_id as i32
            )
            .execute(db_pool)
            .await
            .unwrap();
            let zone_population_id = sqlx::query!(
                "INSERT INTO zone_population (zone_id, world_population_id) VALUES ($1, $2) RETURNING zone_population_id",
                *zone_id as i32,
                world_population_id
            ).fetch_one(db_pool).await.unwrap().zone_population_id;


            for (faction_id, team_map) in faction_map.iter() {
                for (team_id, loadout_map) in team_map.iter() {
                    sqlx::query!(
                        "INSERT INTO faction (faction_id) VALUES ($1) ON CONFLICT DO NOTHING",
                        *faction_id as i32
                    )
                    .execute(db_pool)
                    .await
                    .unwrap();
                    sqlx::query!(
                        "INSERT INTO faction (faction_id) VALUES ($1) ON CONFLICT DO NOTHING",
                        *team_id as i32
                    )
                    .execute(db_pool)
                    .await
                    .unwrap();

                    let faction_population_id = sqlx::query!(
                        "INSERT INTO faction_population (faction_id, team_id, zone_population_id) VALUES ($1, $2, $3) RETURNING faction_population_id",
                        *faction_id as i32,
                        *team_id as i32,
                        zone_population_id
                    ).fetch_one(db_pool).await.unwrap().faction_population_id;

                    for (loadout_id, amount) in loadout_map.iter() {
                        sqlx::query!(
                            "INSERT INTO loadout (loadout_id) VALUES ($1) ON CONFLICT DO NOTHING",
                            *loadout_id as i32
                        )
                        .execute(db_pool)
                        .await
                        .unwrap();
                        sqlx::query!(
                            "INSERT INTO loadout_population (loadout_id, faction_population_id, amount) VALUES ($1, $2, $3)",
                            *loadout_id as i32,
                            faction_population_id,
                            *amount as i32
                        ).execute(db_pool).await.unwrap();
                    }
                }
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
        // store_pop(&loadout_breakdown_numbers, &db_pool).await;
    }
}
