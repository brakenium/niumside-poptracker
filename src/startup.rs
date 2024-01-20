use crate::discord;
use crate::discord::{Data, Error};
use crate::storage::configuration::Settings;
use crate::web::ApiDoc;
use crate::{active_players, census, logging};
use poise::serenity_prelude::ClientBuilder;
use poise::{serenity_prelude, FrameworkBuilder};
use sqlx::PgPool;
use std::thread;
use utoipa::OpenApi;

pub struct DbState {
    pub(crate) pool: PgPool,
}

pub async fn services(
    rocket: rocket::Rocket<rocket::Build>,
    db_pool: PgPool,
    app_config: Settings,
    poise: FrameworkBuilder<Data, Error>,
    active_players: active_players::ActivePlayerDb,
    addr: std::net::SocketAddr,
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
    let poise_framework = poise
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(discord::Data { db_pool: poise_db })
            })
        })
        .build();

    let intents = serenity_prelude::GatewayIntents::non_privileged();
    let mut poise_client = ClientBuilder::new(app_config.discord.token, intents)
        .framework(poise_framework)
        .await
        .expect("Failed to create Discord client");

    let census_realtime_state = census::realtime::State {
        active_players: active_players.clone(),
    };

    let census_realtime_config = census::realtime::RealtimeClientConfig {
        environment: "ps2".to_owned(),
        service_id: app_config.census.service_id,
        realtime_url: Some(app_config.census.realtime_base_url),
    };

    thread::spawn(move || census::realtime::client(census_realtime_config, &census_realtime_state));

    tokio::select!(
        _ = poise_client.start() => {},
        _ = rocket.launch() => {},
        _ = active_players::process_loop(active_players.clone(), db_pool) => {},
        _ = active_players::clean(active_players.clone()) => {},
    );

    Ok(())
}
