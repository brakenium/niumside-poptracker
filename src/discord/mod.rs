mod commands;
#[cfg(feature = "census")]
mod census_formatting;
#[cfg(feature = "census")]
mod icons;
mod updaters;
mod formatting;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use poise::{FrameworkBuilder};
use sqlx::PgPool;
use crate::storage::configuration::{DiscordCalendarConfig, GoogleConfig};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{EventHandler, FullEvent, GuildId};
use crate::discord::updaters::Updater;

pub struct Data {
    pub(crate) db_pool: PgPool,
    pub(crate) google: GoogleConfig,
    pub(crate) calendar: DiscordCalendarConfig
} // User data, which is stored and accessible in all command invocations

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

async fn event_handler(
    ctx: &serenity::Context,
    event: &FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    let ctx = Arc::new(ctx.clone());

    match event {
        FullEvent::CacheReady {..} => {
            let ctx1 = Arc::clone(&ctx);

            tokio::spawn(async move {
                loop {
                    updaters::update_calendar::UpdateCalendar::update(&ctx1).await.unwrap();

                    tokio::time::sleep(tokio::time::Duration::from_secs(15 * 60)).await;
                }
            });
        },
        _ => {}
    }

    Ok(())
}

pub fn init() -> FrameworkBuilder<Data, Error> {
    poise::Framework::builder().options(poise::FrameworkOptions { commands: vec![
            #[cfg(feature = "census")]
            commands::census::population(),
            commands::generic::age(),
        ], event_handler: |ctx, event, framework, data| {

            Box::pin(event_handler(ctx, event, framework, data))
        }, ..Default::default() })
}
