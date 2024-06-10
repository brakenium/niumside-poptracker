use crate::discord;
use poise::serenity_prelude as serenity;
use crate::discord::Data;

pub mod update_calendar;
mod utils;

pub trait Updater {
    async fn update(ctx: &serenity::Context, data: &Data) -> Result<(), discord::Error>;
}
