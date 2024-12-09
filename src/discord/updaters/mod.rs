use crate::discord;
use crate::discord::Data;
use poise::serenity_prelude as serenity;

pub mod membership_reminder;
pub mod update_calendar;
mod utils;

pub trait Updater {
    async fn update(ctx: &serenity::Context, data: &Data) -> Result<(), discord::Error>;
}
