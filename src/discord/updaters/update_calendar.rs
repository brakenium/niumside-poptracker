use chrono::Utc;
use crate::{discord, google_calendar};
use crate::discord::updaters::Updater;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Colour, CreateEmbed, EditMessage};
use crate::discord::{Data, formatting};
use crate::discord::updaters::utils::get_message_or_create_new;
use crate::google_calendar::get_calendar_color;

pub struct UpdateCalendar;

impl Updater for UpdateCalendar {
    async fn update(ctx: &serenity::Context, data: &Data) -> Result<(), discord::Error> {

        let events = google_calendar::get_next_week(
            &data.google,
            &data.calendar.google_calendar_id
        ).await.ok_or("Failed to get google calendar events")?;

        let mut embeds: Vec<CreateEmbed> = Vec::new();

        for event in events.items.unwrap_or_default() {
            let color_string = match google_calendar::get_event_color(&data.google, &event).await {
                Some(color) => color,
                None => {
                    get_calendar_color(&data.google, &data.calendar.google_calendar_id).await.unwrap_or_else(|| "000000".to_string())
                }
            };

            let mut color = Colour::default();
            if let Ok(color_int) = u32::from_str_radix(&color_string[1..], 16) {
                color.0 = color_int;
            };
            embeds.push(formatting::calendar_event(&event, color, Utc::now()));
        }

        let mut message = get_message_or_create_new(ctx, data.calendar.channel_id, data.calendar.message_id).await?;

        let message_content = EditMessage::new()
            .content("")
            .embeds(embeds);

        message.edit(ctx, message_content).await?;

        Ok(())
    }
}