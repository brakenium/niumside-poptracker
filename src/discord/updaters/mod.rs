use std::sync::Arc;
use crate::discord;
use poise::serenity_prelude as serenity;

pub mod update_calendar;
mod utils;

pub(crate) trait Updater {
    async fn update(ctx: &serenity::Context) -> Result<(), discord::Error>;
}
