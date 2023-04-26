#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

mod configuration;
mod constants;
mod realtime;
mod event_handlers;
mod active_players;
mod storage;
mod logging;
use futures::future;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app_config = configuration::Settings::new()?;

    logging::init(&app_config);

    // write a match expression for realtime::init(app_config.census, app_config.worlds).await
    // if events is Ok, then do the following
    // if events is Err, then do the following
    let events = match realtime::init(app_config.census, app_config.worlds).await {
        Ok(events) => events,
        Err(e) => {
            panic!("Unable to connect to realtime API: {}", e);
        }
    };

    let Ok(db_pool) = storage::pool::create(&app_config.database.connection_string).await
    else {
        panic!("Unable to connect to database");
    };

    let active_players: active_players::ActivePlayerDb = Arc::new(Mutex::new(HashMap::new()));

    let futures = vec![
        tokio::spawn(active_players::process_loop(active_players.clone(), db_pool)),
        tokio::spawn(event_handlers::receive_events(events, active_players.clone())),
        tokio::spawn(active_players::clean(active_players.clone()))
    ];

    future::join_all(futures).await;
    Ok(())
}
