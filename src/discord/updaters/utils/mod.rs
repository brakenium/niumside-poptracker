use poise::{serenity_prelude as serenity};
use poise::serenity_prelude::{ChannelId, CreateMessage, GetMessages, Message, MessageId};
use tracing::log::error;
use crate::discord;

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