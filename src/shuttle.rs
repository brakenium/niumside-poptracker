use sqlx::PgPool;
use crate::active_players;
use crate::event_handlers;
use crate::logging;
use crate::realtime;
use auraxis::realtime::event::Event;
use tokio::sync::mpsc::Receiver;

pub struct NiumsideService {
    pub(crate) active_players: active_players::ActivePlayerDb,
    pub(crate) db_pool: PgPool,
    pub(crate) secrets: shuttle_secrets::SecretStore,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for NiumsideService {
    async fn bind(
        mut self,
        addr: std::net::SocketAddr,
    ) -> Result<(), shuttle_runtime::Error> {
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
            _ = active_players::process_loop(self.active_players.clone(), self.db_pool) => {},
            _ = event_handlers::receive_events(events, self.active_players.clone()) => {},
            _ = active_players::clean(self.active_players.clone()) => {},
        );

        Ok(())
    }
}
