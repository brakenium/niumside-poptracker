#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

mod active_players;
mod event_handlers;
mod logging;
mod realtime;
mod shuttle;
mod web;
mod discord;
mod controllers;

use shuttle_runtime::CustomError;
use sqlx::PgPool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::path::PathBuf;

#[shuttle_runtime::main]
async fn init(
    #[shuttle_shared_db::Postgres] postgres: PgPool,
    #[shuttle_secrets::Secrets] secrets: shuttle_secrets::SecretStore,
    #[shuttle_static_folder::StaticFolder(folder = "swagger-v4.19.0")] swagger: PathBuf,
) -> Result<shuttle::NiumsideService, shuttle_runtime::Error> {
    sqlx::migrate!()
        .run(&postgres.clone())
        .await
        .map_err(CustomError::new)?;

    let active_players: active_players::ActivePlayerDb = Arc::new(Mutex::new(HashMap::new()));

    let rocket = web::init(swagger);

    let poise = discord::init(&secrets.get("DISCORD_TOKEN").unwrap());

    Ok(shuttle::NiumsideService {
        active_players,
        db_pool: postgres,
        secrets,
        rocket,
        poise,
    })
}
