mod configuration;
mod constants;
mod realtime;
mod event_handlers;
mod active_players;
mod storage;
use futures::future;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let app_config = configuration::Settings::new()?;

    let Ok(events) = realtime::init(app_config.census, app_config.worlds).await
    else {
        panic!("Unable to connect to Census realtime API");
    };

    let Ok(db_pool) = storage::pool::create(&app_config.database.connection_string).await
    else {
        panic!("Unable to connect to database");
    };

    let active_players: active_players::ActivePlayerDb = Arc::new(Mutex::new(HashMap::new()));

    let futures = vec![
        tokio::spawn(active_players::process_loop(active_players.clone(), db_pool)),
        tokio::spawn(event_handlers::receive_events(events, active_players.clone())),
        tokio::spawn(active_players::clean_active_players(active_players.clone()))
    ];

    future::join_all(futures).await;
    Ok(())
}
