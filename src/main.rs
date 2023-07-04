#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

mod active_players;
mod event_handlers;
mod logging;
mod realtime;
#[cfg(not(feature = "standalone"))]
mod shuttle;
mod web;
mod discord;
mod controllers;
mod constants;
mod storage;
mod startup;

use sqlx::PgPool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::path::PathBuf;
use poise::FrameworkBuilder;
use crate::storage::configuration::Settings;

struct Services {
    active_players: active_players::ActivePlayerDb,
    db_pool: PgPool,
    rocket: rocket::Rocket<rocket::Build>,
    poise: FrameworkBuilder<discord::Data, discord::Error>,
}

async fn agnostic_init(postgres: PgPool, swagger: PathBuf, app_config: Settings) -> anyhow::Result<Services> {
    sqlx::migrate!()
        .run(&postgres.clone())
        .await?;

    let active_players: active_players::ActivePlayerDb = Arc::new(Mutex::new(HashMap::new()));

    let rocket = web::init(swagger);

    let poise = discord::init(&app_config.discord.token);

    Ok(Services {
        active_players,
        db_pool: postgres,
        rocket,
        poise,
    })
}

#[cfg(not(feature = "standalone"))]
#[shuttle_runtime::main]
async fn init(
    #[shuttle_shared_db::Postgres] postgres: PgPool,
    #[shuttle_static_folder::StaticFolder(folder = "swagger-v4.19.0")] swagger: PathBuf,
    #[shuttle_static_folder::StaticFolder(folder = "config")] config_folder: PathBuf,
) -> Result<shuttle::NiumsideService, shuttle_runtime::Error> {
    let app_config = Settings::new(config_folder).map_err(|e| anyhow::Error::new(e))?;

    let initialised_services = agnostic_init(postgres, swagger, app_config.clone()).await?;

    Ok(shuttle::NiumsideService {
        active_players: initialised_services.active_players,
        db_pool: initialised_services.db_pool,
        app_config,
        rocket: initialised_services.rocket,
        poise: initialised_services.poise,
    })
}

#[cfg(feature = "standalone")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_config = Settings::new(PathBuf::from("config"))?;

    logging::tracing(app_config.app.log_level);

    let postgres = storage::db_pool::create(&app_config.database.connection_string.clone()).await?;

    let initialised_services = agnostic_init(
        postgres,
        PathBuf::from("swagger-v4.19.0"),
        app_config.clone()
    ).await?;

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8000));

    startup::services(
        initialised_services.rocket,
        initialised_services.db_pool,
        app_config,
        initialised_services.poise,
        initialised_services.active_players,
        addr,
    ).await?;

    Ok(())
}