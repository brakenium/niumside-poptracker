use crate::{active_players, startup};
use sqlx::PgPool;
use crate::discord::{Data, Error};
use crate::storage::configuration;

pub struct NiumsideService {
    pub(crate) active_players: active_players::ActivePlayerDb,
    pub(crate) db_pool: PgPool,
    pub(crate) app_config: configuration::Settings,
    pub(crate) rocket: rocket::Rocket<rocket::Build>,
    pub(crate) poise: poise::FrameworkBuilder<Data, Error>,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for NiumsideService {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        Box::pin(startup::services(
            self.rocket,
            self.db_pool,
            self.app_config,
            self.poise,
            self.active_players,
            addr
        )).await?;

        Ok(())
    }
}
