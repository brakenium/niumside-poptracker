use crate::{active_players, event_handlers, logging, realtime};
use sqlx::PgPool;

pub struct DbState {
    pub(crate) pool: PgPool,
}

pub struct NiumsideService {
    pub(crate) active_players: active_players::ActivePlayerDb,
    pub(crate) db_pool: PgPool,
    pub(crate) secrets: shuttle_secrets::SecretStore,
    pub(crate) rocket: rocket::Rocket<rocket::Build>,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for NiumsideService {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let shutdown = rocket::config::Shutdown {
            ctrlc: false,
            ..rocket::config::Shutdown::default()
        };

        let config = self
            .rocket
            .figment()
            .clone()
            .merge((rocket::Config::ADDRESS, addr.ip()))
            .merge((rocket::Config::PORT, addr.port()))
            .merge((rocket::Config::LOG_LEVEL, rocket::config::LogLevel::Off))
            .merge((rocket::Config::SHUTDOWN, shutdown));

        let db_state = DbState {
            pool: self.db_pool.clone(),
        };
        let rocket = self
            .rocket
            .configure(config)
            .manage(logging::metrics())
            .manage(db_state);

        // write a match expression for realtime::init(app_config.census, app_config.worlds).await
        // if events is Ok, then do the following
        // if events is Err, then do the following
        let events = match realtime::init(self.secrets).await {
            Ok(events) => events,
            Err(e) => {
                panic!("Unable to connect to realtime API: {e}");
            }
        };

        tokio::select!(
            _ = rocket.launch() => {},
            _ = active_players::process_loop(self.active_players.clone(), self.db_pool) => {},
            _ = event_handlers::receive_events(events, self.active_players.clone()) => {},
            _ = active_players::clean(self.active_players.clone()) => {},
        );

        Ok(())
    }
}
