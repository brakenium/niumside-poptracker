#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

mod constants;
mod realtime;
mod event_handlers;
mod active_players;
mod logging;
mod shuttle;

use futures::future;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use sqlx::{Executor, PgPool};
use shuttle_runtime::CustomError;

#[shuttle_runtime::main]
async fn init(
    #[shuttle_shared_db::Postgres] postgres: PgPool,
    #[shuttle_secrets::Secrets] secrets: shuttle_secrets::SecretStore,
) -> Result<shuttle::NiumsideService, shuttle_runtime::Error> {
    sqlx::migrate!().run(&postgres.clone())
        .await
        .map_err(CustomError::new)?;

    let active_players: active_players::ActivePlayerDb = Arc::new(Mutex::new(HashMap::new()));

    Ok(shuttle::NiumsideService {
        active_players,
        db_pool: postgres,
        secrets,
    })
}
