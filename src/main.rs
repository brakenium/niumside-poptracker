#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::module_name_repetitions)]

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
mod logging;
#[cfg(feature = "census")]
mod serde;
mod startup;
mod storage;
mod web;
mod google_calendar;

use crate::discord::{Data, Error};
use crate::storage::configuration::Settings;
use poise::FrameworkBuilder;
use sqlx::PgPool;
use std::path::Path;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

struct Services {
    #[cfg(feature = "census")]
    active_players: active_players::ActivePlayerDb,
    db_pool: PgPool,
    rocket: rocket::Rocket<rocket::Build>,
    poise: FrameworkBuilder<Data, Error>,
}

async fn agnostic_init(postgres: PgPool) -> anyhow::Result<Services> {
    sqlx::migrate!().run(&postgres.clone()).await?;

    #[cfg(feature = "census")]
    let active_players: active_players::ActivePlayerDb = Arc::new(Mutex::new(HashMap::new()));

    let rocket = web::init();

    let poise = discord::init();

    Ok(Services {
        #[cfg(feature = "census")]
        active_players,
        db_pool: postgres,
        rocket,
        poise,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_config = Settings::new(Path::new("config"))?;

    logging::tracing(app_config.app.log_level);

    let postgres = storage::db_pool::create(&app_config.database.connection_string.clone()).await?;

    let initialised_services = agnostic_init(postgres).await?;

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8000));

    Box::pin(startup::services(
        initialised_services.rocket,
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
