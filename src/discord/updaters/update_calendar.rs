use calendar3::api::{Event, EventDateTime};
use chrono::Utc;
use crate::{discord, google_calendar};
use crate::discord::updaters::Updater;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Colour, CreateAttachment, CreateEmbed, CreateScheduledEvent, EditMessage, User};
use poise::serenity_prelude::ScheduledEventType::External;
use tracing::{debug, error, info, trace};
use crate::discord::{Data, formatting};
use crate::discord::updaters::utils::get_message_or_create_new;
use crate::google_calendar::get_calendar_color;

pub struct UpdateCalendar;

struct ToScheduleEvent<'a> {
    event: CreateScheduledEvent<'a>,
    google_event_id: String,
}

impl Updater for UpdateCalendar {
    async fn update(ctx: &serenity::Context, data: &Data) -> Result<(), discord::Error> {
        let events = match google_calendar::get_next_week(
            &data.google,
            &data.calendar.google_calendar_id
        ).await {
            None => {
                let error = Err(discord::Error::from("Failed to get events"));
                error!("Failed to get events: {:?}", error);
                return error;
            }
            Some(events) => events
        };

        let mut embeds: Vec<CreateEmbed> = Vec::new();

        let mut to_schedule_events: Vec<ToScheduleEvent> = Vec::new();

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

            let start_date_time = match event.start.as_ref() {
                None => {
                    error!("Failed to get start time for event: {:?}", event);
                    continue;
                }
                Some(start) => {
                    let Some(start_date_time) = start.date_time.as_ref() else {
                        error!("Failed to get start date time for event: {:?}", event);
                        continue;
                    };

                    start_date_time
                }
            };

            let end_date_time = match event.end.as_ref() {
                None => {
                    error!("Failed to get end time for event: {:?}", event);
                    continue;
                }
                Some(end) => {
                    let Some(end_date_time) = end.date_time.as_ref() else {
                        error!("Failed to get end date time for event: {:?}", event);
                        continue;
                    };

                    end_date_time
                }
            };

            let Some(summary) = event.summary.as_ref() else {
                error!("Failed to get summary for event: {:?}", event);
                continue;
            };

            let description = event.description.as_ref()
                .map_or_else(|| "No description".to_string(),
                             |description| google_calendar::formatting::html_to_md(description)
                );

            let Some(location) = event.location.as_ref() else {
                error!("Failed to get location for event: {:?}", event);
                continue;
            };

            let Some(event_id) = event.id.as_ref() else {
                error!("Failed to get event ID for event: {:?}", event);
                continue;
            };

            let image_attachment = {
                let payload = steganography::util::str_to_bytes(event_id);

                let Some(image_path) = data.calendar.image_path.to_str() else {
                    error!("Failed to get image path for event: {:?}", event);
                    continue;
                };

                let image = steganography::util::file_as_dynamic_image(image_path.to_string());

                let image = steganography::encoder::Encoder::new(payload, image).encode_alpha().into_vec();

                CreateAttachment::bytes(image, "image.png".to_string())
            };

            let scheduled_event = CreateScheduledEvent::new(
                External,
                summary,
                *start_date_time,
            )
                .image(&image_attachment)
                .location(location)
                .start_time(*start_date_time)
                .end_time(*end_date_time)
                .description(description);

            to_schedule_events.push(ToScheduleEvent {
                event: scheduled_event,
                google_event_id: event_id.clone(),
            });
        }

        let mut message = get_message_or_create_new(ctx, data.calendar.channel_id, data.calendar.message_id).await?;

        let message_content = EditMessage::new()
            .content("")
            .embeds(embeds);

        message.edit(ctx, message_content).await?;

        let scheduled_events = data.calendar.guild_id.scheduled_events(ctx.http.clone(), false).await?;
        
        let events_scheduled_by_bot = scheduled_events.iter().filter(|event| {
            let default_user = User::default();
            let creator = event.creator.as_ref().unwrap_or_else(|| &default_user);
            
            creator.id == ctx.cache.current_user().id
        }).collect::<Vec<_>>();

        // for event in scheduled_events {
        //     let Some(image_hash) = event.image.as_ref() else {
        //         continue;
        //     };
        //     
        //     let event_id = event.id;
        //     
        //     let url = format!("https://cdn.discordapp.com/guild-events/{event_id}/{image_hash}");
        //     
        //     let image = reqwest::get(url).await?;
        // }

        for single_to_schedule in to_schedule_events {
            data.calendar.guild_id.create_scheduled_event(ctx, single_to_schedule.event).await?;
        }

        Ok(())
    }
}