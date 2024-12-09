#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::module_name_repetitions)]
#![allow(dead_code)]

extern crate google_calendar3 as calendar3;

#[cfg(feature = "census")]
mod active_players;
#[cfg(feature = "census")]
mod census;
mod constants;
#[cfg(feature = "census")]
mod controllers;
mod discord;
#[cfg(feature = "census")]
mod event_handlers;
mod google_calendar;
mod logging;
#[cfg(feature = "census")]
mod serde;
mod startup;
mod storage;
mod utils;
mod web;

use crate::active_players::ActivePlayerHashmap;
use crate::discord::{Data, Error};
use crate::storage::configuration::Settings;
use poise::FrameworkBuilder;
#[cfg(feature = "database")]
use sqlx::PgPool;
use std::path::Path;
use std::sync::{Arc, Mutex};

struct Services {
    #[cfg(feature = "census")]
    active_players: active_players::ActivePlayerDb,
    #[cfg(feature = "database")]
    db_pool: PgPool,
    rocket: rocket::Rocket<rocket::Build>,
    poise: FrameworkBuilder<Data, Error>,
}

#[allow(clippy::unused_async)]
async fn agnostic_init(#[cfg(feature = "database")] postgres: PgPool) -> anyhow::Result<Services> {
    #[cfg(feature = "census")]
    let active_players: active_players::ActivePlayerDb =
        Arc::new(Mutex::new(ActivePlayerHashmap::new()));

    let rocket = web::init();

    let poise = discord::init();

    Ok(Services {
        #[cfg(feature = "census")]
        active_players,
        #[cfg(feature = "database")]
        db_pool: postgres,
        rocket,
        poise,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = Settings::new(Path::new("config"))?;

    logging::tracing(app_config.app.log_level);

    #[cfg(feature = "database")]
    let postgres = storage::db_pool::create(&app_config.database.connection_string.clone()).await?;

    let initialised_services = agnostic_init(
        #[cfg(feature = "database")]
        postgres,
    )
        .await?;

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8000));

    Box::pin(startup::services(
        initialised_services.rocket,
        #[cfg(feature = "database")]
        initialised_services.db_pool,
        app_config,
        initialised_services.poise,
        #[cfg(feature = "census")]
        initialised_services.active_players,
        addr,
    ))
        .await?;

    Ok(())
}
