pub mod gain_experience;
use auraxis::realtime::event::Event;
use tokio::sync::mpsc::Receiver;

use crate::{active_players::ActivePlayerDb, storage};

#[derive(thiserror::Error, Debug)]
pub enum EventHandlerErrors {
    #[error("An error occurred while interacting with the database for a stream event")]
    SqlxError(#[from] sqlx::Error),
}

pub async fn receive_events(mut events: Receiver<Event>, active_players: ActivePlayerDb) -> Option<()> {
    while let Some(event) = events.recv().await {
        let active_players = active_players.clone();
        tokio::spawn(async move {
            let error = match event {
                Event::GainExperience(event) => gain_experience::handle(event, active_players).await,
                _ => Ok(()),
            };

            match error {
                Ok(_) => (),
                Err(error) => tracing::error!("{}", error),
            };
        });
    }
    Some(())
}

