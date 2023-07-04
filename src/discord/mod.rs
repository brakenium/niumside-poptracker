mod commands;
mod formatting;
mod icons;

use poise::FrameworkBuilder;
use poise::serenity_prelude as serenity;
use sqlx::PgPool;

pub struct Data {
    pub(crate) db_pool: PgPool,
} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub fn init(discord_token: &str) -> FrameworkBuilder<Data, Error> {
    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::population()],
            ..Default::default()
        })
        .intents(serenity::GatewayIntents::non_privileged())
        .token(discord_token)
}