pub mod gain_experience;
use crate::active_players::ActivePlayerDb;
use census::realtime::event::Event;
use tokio::sync::mpsc::Receiver;
use tracing::error;
use census::realtime::client::{RealtimeClient, WsReceive};

#[derive(thiserror::Error, Debug)]
pub enum EventHandlerErrors {
    #[error("An error occurred while interacting with the database for a stream event")]
    SqlxError(#[from] sqlx::Error),
}

pub async fn receive_events(
    mut client: RealtimeClient,
    active_players: ActivePlayerDb,
) -> Option<()> {
    loop {
        match client.ws_receive.recv().await {
            Some(event) => {
                let active_players = active_players.clone();
                tokio::spawn(async move {
                    match event {
                        Event::GainExperience(event) => {
                            gain_experience::handle(&event, &active_players);
                        }
                        Event::PlayerLogin(_) => todo!(),
                        Event::PlayerLogout(_) => todo!(),
                        Event::Death(_) => todo!(),
                        Event::VehicleDestroy(_) => todo!(),
                        Event::PlayerFacilityCapture(_) => todo!(),
                        Event::PlayerFacilityDefend(_) => todo!(),
                        Event::ContinentLock(_) => todo!(),
                        Event::ContinentUnlock(_) => todo!(),
                        Event::FacilityControl(_) => todo!(),
                        Event::MetagameEvent(_) => todo!(),
                        Event::ItemAdded => todo!(),
                        Event::AchievementEarned => todo!(),
                        Event::SkillAdded => todo!(),
                        Event::BattleRankUp => todo!(),
                    };
                });
            }
            None => return None,
        };
    }
}
