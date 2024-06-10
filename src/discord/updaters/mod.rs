use crate::discord;
use poise::serenity_prelude as serenity;

pub mod update_calendar;
mod utils;

pub trait Updater {
    async fn update(ctx: &serenity::Context) -> Result<(), discord::Error>;
}
