use poise::{serenity_prelude as serenity, serenity_prelude};
use poise::serenity_prelude::{ChannelId, CreateMessage, GetMessages, GuildChannel, Message, MessageId};
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

            match guild_channel.messages(ctx, get_messages).await {
                Ok(messages) => {
                    let messages = messages.clone();
                    if let Some(message) = messages.first() {
                        message.clone()
                    } else {
                        guild_channel.send_message(ctx, empty_message).await?
                    }
                }
                Err(_) => {
                    guild_channel.send_message(ctx, empty_message).await?
                }
            }
        }
        None => {
            guild_channel.send_message(ctx, empty_message).await?
        }
    };

    Ok(message)
}