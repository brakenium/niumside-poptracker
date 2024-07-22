use calendar3::api::{Event, EventDateTime};
use chrono::{DateTime, Utc};
use crate::{discord, google_calendar};
use crate::discord::updaters::Updater;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Colour, CreateAttachment, CreateEmbed, CreateScheduledEvent, EditMessage, EditScheduledEvent, Error, ModelError, ScheduledEventId, Timestamp, User};
use poise::serenity_prelude::ScheduledEventType::External;
use sqlx::types::BigDecimal;
use tracing::{debug, error, info, trace};
use crate::discord::{Data, formatting};
use crate::discord::updaters::utils::{create_or_edit_event, get_message_or_create_new, ToScheduleEventFields};
use crate::google_calendar::formatting::html_to_md;
use crate::google_calendar::get_calendar_color;
use crate::storage::configuration::DiscordCalendarConfig;

pub struct UpdateCalendar;

struct ToScheduleEvent {
    event_fields: ToScheduleEventFields,
    calendar_events_id: i32,
}

async fn update_single_calendar(ctx: &serenity::Context, data: &Data, calendar: &DiscordCalendarConfig) -> Result<(), discord::Error> {
    let events = match google_calendar::get_next_week(
        &data.google,
        &calendar.google_calendar_id
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
                get_calendar_color(&data.google, &calendar.google_calendar_id).await.unwrap_or_else(|| "000000".to_string())
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

                *start_date_time
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

                *end_date_time
            }
        };

        let title = match event.summary.as_ref() {
            None => {
                error!("Failed to get event summary for event, using fallback title: {:?}", event);
                String::from("Couldn't get title from calendar event")
            }
            Some(summary) => summary.clone()
        };

        let description = match event.description.as_ref() {
            None => None,
            Some(desc) => Some(html_to_md(desc))
        };

        let Some(event_id) = event.id.as_ref() else {
            error!("Failed to get event ID for event: {:?}", event);
            continue;
        };

        let event_fields = ToScheduleEventFields {
            title,
            start_date_time,
            end_date_time,
            location: event.location,
            description,
        };

        let calendar_event_insert = sqlx::query!(
                "INSERT INTO calendar_events (
                    calendar_id,
                    calendar_event_id
                )
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING",
                calendar.google_calendar_id,
                event_id,
            )
            .execute(&data.db_pool)
            .await;

        match calendar_event_insert {
            Ok(_) => {
                let calendar_events_id = sqlx::query!(
                        "SELECT calendar_events_id FROM calendar_events
                        WHERE
                            calendar_id = $1 AND
                            calendar_event_id = $2",
                        calendar.google_calendar_id,
                        event_id,
                    )
                    .fetch_one(&data.db_pool)
                    .await?
                    .calendar_events_id;

                to_schedule_events.push(ToScheduleEvent {
                    event_fields,
                    calendar_events_id,
                });
            }
            Err(error) => {
                error!("Failed to insert google calendar event into database: {:?}", error);
            }
        }
    }

    let mut message = get_message_or_create_new(ctx, calendar.channel_id, calendar.message_id).await?;

    let message_content = EditMessage::new()
        .content("")
        .embeds(embeds);

    message.edit(ctx, message_content).await?;

    let scheduled_events = calendar.guild_id.scheduled_events(ctx.http.clone(), false).await?;

    let events_scheduled_by_bot = scheduled_events.iter().filter(|event| {
        let default_user = User::default();
        let creator = event.creator.as_ref().unwrap_or_else(|| &default_user);

        creator.id == ctx.cache.current_user().id
    }).collect::<Vec<_>>();

    for single_to_schedule in to_schedule_events {
        let guild_id = calendar.guild_id.get();

        let database_record = sqlx::query!(
                "SELECT discord_id, CE.calendar_events_id FROM calendar_events AS CE
                LEFT JOIN discord_events AS DE ON DE.calendar_events_id = CE.calendar_events_id
                WHERE
                    CE.calendar_id = $1 AND
                    CE.calendar_events_id = $2 AND
                    DE.guild_id = $3",
                calendar.google_calendar_id,
                single_to_schedule.calendar_events_id,
                guild_id as i64,
            ).fetch_optional(&data.db_pool).await?;

        if let Some(database_record) = database_record {
            let discord_id = database_record.discord_id;

            let event = events_scheduled_by_bot.iter().find(|event| event.id == discord_id as u64);

            let discord_event = match create_or_edit_event(ctx, calendar, event.copied(), &single_to_schedule.event_fields).await {
                Ok(discord_event) => discord_event,
                Err(error) => {
                    error!("Failed to create or edit event: {:?}", error);
                    continue;
                }
            };

            sqlx::query!(
                    "INSERT INTO discord_events (
                        calendar_events_id,
                        guild_id,
                        discord_id
                    )
                    VALUES ($1, $2, $3)
                    ON CONFLICT DO NOTHING",
                    database_record.calendar_events_id,
                    guild_id as i64,
                    discord_event.id.get() as i64,
                )
                .execute(&data.db_pool)
                .await?;
        }
        else {
            let discord_event = match create_or_edit_event(ctx, calendar, None, &single_to_schedule.event_fields).await {
                Ok(discord_event) => discord_event,
                Err(error) => {
                    error!("Failed to create or edit event: {:?}", error);
                    continue;
                }
            };

            sqlx::query!(
                    "INSERT INTO discord_events (
                        calendar_events_id,
                        guild_id,
                        discord_id
                    )
                    VALUES ($1, $2, $3)
                    ON CONFLICT DO NOTHING",
                    single_to_schedule.calendar_events_id,
                    guild_id as i64,
                    discord_event.id.get() as i64,
                )
                .execute(&data.db_pool)
                .await?;
        }
    }

    Ok(())
}


impl Updater for UpdateCalendar {

    async fn update(ctx: &serenity::Context, data: &Data) -> Result<(), discord::Error> {
        for calendar in &data.calendar {
            let result = update_single_calendar(ctx, data, calendar).await;

            if let Err(error) = result {
                error!("Failed to update calendar: {:?}", error);
            }
        }

        Ok(())
    }
}