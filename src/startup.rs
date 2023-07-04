
use sqlx::PgPool;
use utoipa::OpenApi;
use crate::{active_players, event_handlers, logging, realtime};
use crate::storage::configuration::Settings;
use crate::web::ApiDoc;
use crate::discord;

pub struct DbState {
    pub(crate) pool: PgPool,
}

pub async fn services(
    rocket: rocket::Rocket<rocket::Build>,
    db_pool: PgPool,
    app_config: Settings,
    poise: poise::FrameworkBuilder<discord::Data, discord::Error>,
    active_players: active_players::ActivePlayerDb,
    addr: std::net::SocketAddr
) -> anyhow::Result<()> {
    let shutdown = rocket::config::Shutdown {
        ctrlc: false,
        ..rocket::config::Shutdown::default()
    };

    let db_state = DbState {
        pool: db_pool.clone(),
    };

    let config = rocket
        .figment()
        .clone()
        .merge((rocket::Config::ADDRESS, addr.ip()))
        .merge((rocket::Config::PORT, addr.port()))
        .merge((rocket::Config::LOG_LEVEL, rocket::config::LogLevel::Off))
        .merge((rocket::Config::SHUTDOWN, shutdown));

    let rocket = rocket
        .configure(config)
        .manage(logging::metrics())
        .manage(db_state)
        .manage(ApiDoc::openapi());

    let poise_db = db_pool.clone();
    let poise = poise.setup(|ctx, _ready, framework| {
        Box::pin(async move {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(discord::Data {
                db_pool: poise_db,
            })
        })
    });

    let events = match realtime::init(app_config.census, app_config.worlds).await {
        Ok(events) => events,
        Err(e) => {
            panic!("Unable to connect to realtime API: {e}");
        }
    };

    tokio::select!(
            _ = poise.run() => {},
            _ = rocket.launch() => {},
            _ = active_players::process_loop(active_players.clone(), db_pool) => {},
            _ = event_handlers::receive_events(events, active_players.clone()) => {},
            _ = active_players::clean(active_players.clone()) => {},
        );

    Ok(())
}