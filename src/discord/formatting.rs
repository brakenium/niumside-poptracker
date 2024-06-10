use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter, FormattedTimestamp};
use calendar3::api::Event;
use chrono::Utc;
use crate::google_calendar::formatting::html_to_md;

pub fn calendar_event(
    event: &Event,
    color: Colour,
    timestamp: chrono::DateTime<Utc>,
) -> CreateEmbed {
    let start = event.start.as_ref().unwrap().date_time.unwrap_or_default();
    let formatted_start = FormattedTimestamp::new(start.into(), None);

    let end = event.end.as_ref().unwrap().date_time.unwrap_or_default();
    let formatted_end = FormattedTimestamp::new(end.into(), None);
    
    let description = html_to_md(&event.description.clone().unwrap_or("No description".to_string()));

    let embed = CreateEmbed::default()
        .title(event.summary.clone().unwrap_or("No title".to_string()))
        .description(description)
        .field("Start", format!("{}", formatted_start), true)
        .field("End", format!("{}", formatted_end), true)
        .color(color)
        // .thumbnail("https://www.planetside2.com/images/ps2-logo.png".to_string())
        .timestamp(timestamp);

    embed
}