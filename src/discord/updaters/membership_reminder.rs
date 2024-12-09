use crate::census::constants::Faction;
use crate::census::rest::client::{CensusRequestableObject, CensusRestClient};
use crate::census::structs::character::{Character, CharacterName, MembershipReminderStatus};
use crate::controllers::character::reset_reminder_for_discord_users;
use crate::discord::formatting::DEFAULT_EMBED_COLOR;
use crate::discord::icons::Icons;
use crate::discord::updaters::Updater;
use crate::discord::{Data, Error};
use chrono::{DateTime, Duration, Utc};
use poise::serenity_prelude::{Context, CreateEmbed, CreateMessage, FormattedTimestamp, FormattedTimestampStyle, Timestamp, User, UserId};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::info;

pub struct MembershipReminder;

// struct UserToRemind {
//     pub discord_user: User,
//     pub characters: Vec<Character>,
// }

// impl UserToRemind {
//     fn new(discord_user: User) -> Self {
//         Self {
//             discord_user,
//             characters: Vec::new(),
//         }
//     }
// }

type UsersToRemind = HashMap<User, Vec<Character>>;

async fn get_users_to_remind(ctx: &Context, db_pool: &PgPool, census_rest_client: &CensusRestClient) -> Result<UsersToRemind, Error> {
    let mut users: UsersToRemind = HashMap::new();

    let characters_to_remind = sqlx::query!(
        "SELECT discord_id, character_id, name, membership_reminder, last_membership_reminder
        FROM planetside_characters
        JOIN users ON planetside_characters.user_id = users.user_id
        WHERE
            membership_reminder = true
            AND discord_id IS NOT NULL"
    )
        .fetch_all(db_pool)
        .await?;

    for char in characters_to_remind {
        let Some(discord_id) = char.discord_id else { continue };

        let user_id = UserId::new(discord_id as u64);
        let discord_user = match ctx.http.get_user(user_id).await {
            Ok(user) => user,
            Err(e) => {
                info!("Failed to get user with id {}: {}", user_id, e);
                continue;
            }
        };

        let last_reminder_time: Option<DateTime<Utc>> = char.last_membership_reminder.map(|last_reminder| last_reminder.and_utc());

        let membership_reminder = MembershipReminderStatus {
            enabled: char.membership_reminder,
            last_reminder: last_reminder_time,
        };

        let mut character = Character::new(char.character_id as u64);

        match character.update_from_rest(census_rest_client).await {
            Ok(()) => (),
            Err(e) => {
                info!("Failed to update character {}: {}", char.character_id, e);
                continue;
            }
        };

        if let Some(times) = &character.times {
            let first_reminder_minimum = Utc::now() - Duration::hours(21);
            let forgotten_reminder_minimum = Utc::now() - Duration::hours(24);

            if times.last_login > first_reminder_minimum {
                continue;
            } else if let Some(last_reminder) = membership_reminder.last_reminder {
                if last_reminder > forgotten_reminder_minimum {
                    continue;
                }
            }
        }

        users.entry(discord_user)
            .or_default()
            .push(character);
    }

    Ok(users)
}

async fn remind_users(ctx: &Context, data: &Data, users: UsersToRemind) -> Result<(), Error> {
    for (usr, characters) in users {
        let mut embed_fields: Vec<(String, String, bool)> = Vec::new();

        for char in &characters {
            // info!("Reminding user {} to log in character {} with faction_id: {}", usr.id, char.character_id, char.faction);
            let wrapped_icons = Icons::try_from(char.faction)
                .unwrap_or(Icons::Ps2White)
                .to_discord_emoji();

            let icon: String = wrapped_icons.map_or_else(
                || char.faction.to_string(),
                |emoji| emoji.to_string(),
            );

            let char_name = &char.name.first;

            let last_login = char.times.as_ref().map_or_else(
                String::new,
                |times| Timestamp::from_unix_timestamp(
                    times.last_login.timestamp()
                )
                    .map_or_else(
                        |_| String::new(),
                        |ts| format!(
                            "{} on {} at {}",
                            FormattedTimestamp::new(ts, FormattedTimestampStyle::RelativeTime.into()),
                            FormattedTimestamp::new(ts, FormattedTimestampStyle::ShortDate.into()),
                            FormattedTimestamp::new(ts, FormattedTimestampStyle::ShortTime.into()),
                        ),
                    ),
            );

            embed_fields.push((
                format!("{icon} {char_name}"),
                format!("Last login: {last_login}"),
                false,
            ));
        }

        let embed = CreateEmbed::default()
            .title("Membership reminder")
            .description("The following characters still need to be logged in to receive their daily login reward:\n")
            .color(DEFAULT_EMBED_COLOR)
            .fields(embed_fields);

        let message = CreateMessage::new()
            .embed(embed);

        match usr.direct_message(ctx, message).await {
            Ok(_) => {
                match reset_reminder_for_discord_users(&data.db_pool, vec![i64::from(usr.id)]).await {
                    Ok(()) => (),
                    Err(e) => {
                        info!("Failed to reset membership reminders for Discord user {}: {}", usr, e);
                        continue;
                    }
                }

                info!("Sent membership reminder to user {}", usr.id);
            }
            Err(e) => info!("Failed to send membership reminder to user {}: {}", usr.id, e),
        };
    }

    Ok(())
}

impl Updater for MembershipReminder {
    async fn update(ctx: &Context, data: &Data) -> Result<(), Error> {
        let users = get_users_to_remind(ctx, &data.db_pool, &data.census_rest_client).await?;
        remind_users(ctx, data, users).await
    }
}