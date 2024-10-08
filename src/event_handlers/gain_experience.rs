use metrics::counter;
use tracing::error;

use crate::active_players::{ActivePlayer, ActivePlayerDb};
use crate::census::event::GainExperience;

pub fn handle(event: &GainExperience, active_players: &ActivePlayerDb) {
    active_players.lock().map_or_else(
        |_| {
            counter!("niumside_active_players_lock_failed").increment(1);
            error!("Unable to lock active players");
        },
        |mut active_players_lock| {
            active_players_lock.insert(
                event.character_id,
                ActivePlayer {
                    zone: event.zone_id,
                    loadout: event.loadout_id,
                    world: event.world_id,
                    last_change: event.timestamp,
                    team_id: event.team_id,
                },
            );
            counter!("niumside_gain_experience_events").increment(1);
        },
    );
}
