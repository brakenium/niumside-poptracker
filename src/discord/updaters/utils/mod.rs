use crate::discord;
use crate::storage::configuration::DiscordCalendarConfig;
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ScheduledEventType::External;
use poise::serenity_prelude::{ChannelId, CreateMessage, CreateScheduledEvent, EditScheduledEvent, GetMessages, Message, MessageId, ScheduledEvent};
use tracing::log::error;

pub async fn get_message_or_create_new(
    ctx: &serenity::Context,
    guild_channel: ChannelId,
    message_id: Option<MessageId>,
) -> Result<Message, discord::Error> {
    let empty_message = CreateMessage::new()
        .content("a");
    let message = match message_id {
        Some(message_id) => {
            let get_messages = GetMessages::new()
                .around(message_id)
                .limit(1);

            let messages = guild_channel.messages(ctx, get_messages).await?;
            messages.first().ok_or_else(|| {
                error!("Failed to get message ({:?}) in channel ({:?})", message_id, guild_channel);
                discord::Error::from("Failed to get message")
            })?.clone()
        }
        None => {
            guild_channel.send_message(ctx, empty_message).await?
        }
    };

    Ok(message)
}

pub struct ToScheduleEventFields {
    pub title: String,
    pub start_date_time: DateTime<Utc>,
    pub end_date_time: DateTime<Utc>,
    pub location: Option<String>,
    pub description: Option<String>,
}

pub async fn create_or_edit_event(ctx: &serenity::Context, calendar: &DiscordCalendarConfig, event: Option<&ScheduledEvent>, event_fields: &ToScheduleEventFields) -> Result<ScheduledEvent, discord::Error> {
    let location = event_fields.location.as_deref().unwrap_or("Unable to get location");

    if let Some(event) = event {
        let event_id = event.id;


        let mut edit_event = EditScheduledEvent::new()
            .name(&event_fields.title)
            .start_time(event_fields.start_date_time)
            .end_time(event_fields.end_date_time)
            .location(location);


        if let Some(description) = &event_fields.description {
            edit_event = edit_event.description(description);
        }

        Ok(calendar.guild_id.edit_scheduled_event(ctx, event_id, edit_event).await?)
    } else {
        let mut create_event = CreateScheduledEvent::new(
            External,
            &event_fields.title,
            event_fields.start_date_time,
        )
            .end_time(event_fields.end_date_time)
            .location(location);

        if let Some(description) = &event_fields.description {
            create_event = create_event.description(description);
        }

        Ok(calendar.guild_id.create_scheduled_event(ctx, create_event).await?)
    }
}