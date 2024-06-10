use poise::serenity_prelude::{Colour, CreateEmbed, FormattedTimestamp};
use calendar3::api::Event;
use chrono::Utc;
use crate::google_calendar::formatting::html_to_md;

pub fn calendar_event(
    event: &Event,
    color: Colour,
    timestamp: chrono::DateTime<Utc>,
) -> CreateEmbed {
    let mut formatted_start = "No start time".to_string();
    if let Some(start_ref) = event.start.as_ref() {
        let start = start_ref.date_time.unwrap_or_default();
        formatted_start = FormattedTimestamp::new(start.into(), None).to_string();
    }

    let mut formatted_end = "No end time".to_string();
    if let Some(end_ref) = event.end.as_ref() {
        let end = end_ref.date_time.unwrap_or_default();
        formatted_end = FormattedTimestamp::new(end.into(), None).to_string();
    }
    
    let description = html_to_md(&event.description.clone().unwrap_or_else(|| "No description".to_string()));

    CreateEmbed::default()
        .title(event.summary.clone().unwrap_or_else(|| "No title".to_string()))
        .description(description)
        .field("Start", formatted_start, true)
        .field("End", formatted_end, true)
        .color(color)
        // .thumbnail("https://www.planetside2.com/images/ps2-logo.png".to_string())
        .timestamp(timestamp)
}