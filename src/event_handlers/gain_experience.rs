use auraxis::realtime::event::GainExperience;
use tracing::error;

use crate::active_players::{ActivePlayerDb, ActivePlayer};

use super::EventHandlerErrors;

pub async fn handle(event: GainExperience, active_players: ActivePlayerDb) -> Result<(), EventHandlerErrors> {
    if let Ok(mut active_players_lock) = active_players.lock() {
        active_players_lock.insert(
            event.character_id,
            ActivePlayer {
                zone: event.zone_id,
                loadout: event.loadout_id,
                world: event.world_id,
                last_change: event.timestamp,
                faction: event.loadout_id.get_faction(),
                team_id: event.team_id,
            },
        );
    } else {
        error!("Unable to lock active players");
    };
    Ok(())
}
