mod configuration;
mod constants;
mod realtime;
mod event_handlers;
mod active_players;
use futures::future;
use realtime::init;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let app_config = configuration::Settings::new()?;

    let Ok(events) = init(app_config.census, app_config.worlds).await
    else {
        panic!("Unable to connect to Census realtime API");
    };

    let active_players: active_players::ActivePlayerDb = Arc::new(Mutex::new(HashMap::new()));

    let futures = vec![
        tokio::spawn(active_players::print_active_players(active_players.clone())),
        tokio::spawn(event_handlers::receive_events(events, active_players.clone())),
        tokio::spawn(active_players::clean_active_players(active_players.clone()))
    ];

    future::join_all(futures).await;
    Ok(())
}
