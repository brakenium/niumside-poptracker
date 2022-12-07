use tracing::info;
use chrono;
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
    pub zone: ZoneID,
    pub loadout: Loadout,
    pub world: WorldID,
    pub last_change: DateTime<Utc>,
}

pub type ActivePlayerDb = Arc<Mutex<HashMap<CharacterID, ActivePlayer>>>;

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

pub async fn print_active_players(active_players: ActivePlayerDb) -> Option<()> {
    let active_players = active_players.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let mut loadout_breakdown: HashMap<WorldID, HashMap<ZoneID, HashMap<Loadout, Vec<ActivePlayer>>>> = HashMap::new();
        let mut loadout_breakdown_numbers: HashMap<WorldID, HashMap<ZoneID, HashMap<Loadout, u16>>> = HashMap::new();

        let active_players_lock = active_players.lock().unwrap();
        active_players_lock.iter().for_each(|(_, player)| {
            loadout_breakdown
                .entry(player.world)
                .or_insert_with(HashMap::new)
                .entry(player.zone)
                .or_insert_with(HashMap::new)
                .entry(player.loadout)
                .or_insert_with(Vec::new)
                .push(player.clone());
            loadout_breakdown_numbers
                .entry(player.world)
                .or_insert_with(HashMap::new)
                .entry(player.zone)
                .or_insert_with(HashMap::new)
                .entry(player.loadout)
                .and_modify(|v| *v += 1)
                .or_insert(1);
        });

        let zone_breakdown: HashMap<WorldID, HashMap<ZoneID, u16>> = loadout_breakdown_numbers.par_iter()
            .map(|(world_id, zones)| {
                let all_world_pop = zones.par_iter().map(|(zone_id, loadouts)| {
                    let all_loadout_pop: Vec<u16> = loadouts.par_iter()
                        .map(|(_, v)| v.clone()).collect();
                    (zone_id.clone(), all_loadout_pop.par_iter().sum::<u16>())
                }).collect();
                (world_id.clone(), all_world_pop)
            }).collect();
        let info_msg = zone_breakdown.par_iter().map(|(world_id, world_pop)| {
            let zone_string = world_pop.par_iter().map(|(zone_id, zone_pop)| format!("\n{}: {}", zone_id, zone_pop)).collect::<String>();
            format!("\n{}:{}\n", world_id, zone_string)
        }).collect::<String>();
        info!("{}", info_msg);
    }
}
