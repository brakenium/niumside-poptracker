use auraxis::realtime::event::GainExperience;

use crate::active_players::{ActivePlayerDb, ActivePlayer};

use super::EventHandlerErrors;

pub async fn handle(event: GainExperience, active_players: ActivePlayerDb) -> Result<(), EventHandlerErrors> {
    let mut active_players_lock = active_players.lock().unwrap();
    active_players_lock.insert(
        event.character_id,
        ActivePlayer {
            zone: event.zone_id,
            loadout: event.loadout_id,
            world: event.world_id,
            last_change: event.timestamp,
        },
    );
    Ok(())
}