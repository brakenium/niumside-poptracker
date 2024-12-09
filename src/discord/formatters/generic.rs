use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;

/// Creates a reply to indicate that the bot is fetching data
/// 
/// # Arguments
/// 
/// * `what_to_await` - The data that the bot is fetching
/// 
/// # Returns
/// 
/// A `CreateReply` with an embed indicating that the bot is fetching data
pub fn fetching_data(what_to_await: &str) -> CreateReply {
    let embed = CreateEmbed::new()
        .title("Fetching data...")
        .description(format!("Fetching {what_to_await} data..."));
    
    CreateReply {
        embeds: vec![embed],
        ..CreateReply::default()
    }
}