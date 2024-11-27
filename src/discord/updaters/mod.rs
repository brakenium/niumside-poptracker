use crate::discord;
use crate::discord::Data;
use poise::serenity_prelude as serenity;

pub mod update_calendar;
mod utils;
pub mod membership_reminder;

pub trait Updater {
    async fn update(ctx: &serenity::Context, data: &Data) -> Result<(), discord::Error>;
}
