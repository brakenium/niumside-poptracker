use crate::census::structs::character::{Character, CharacterName, MembershipReminderStatus};
use sqlx::PgPool;

pub async fn insert_or_update_character(
    pool: &PgPool,
    character: &Character,
    user_id: &i32,
    membership_reminder: &bool,
) -> Result<(), sqlx::Error> {
    #[allow(clippy::cast_possible_wrap)]
    let char_id = character.character_id as i64;
    sqlx::query!(
        "INSERT INTO planetside_characters(
            character_id,
            user_id,
            name,
            membership_reminder,
            faction_id
        ) VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_id, character_id) DO UPDATE
        SET name = $3,
            membership_reminder = $4,
            faction_id = $5",
        char_id,
        user_id,
        character.name.first,
        membership_reminder,
        character.faction as i32
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_characters_by_discord_id(
    pool: &PgPool,
    discord_id: &i64,
) -> Result<Vec<Character>, sqlx::Error> {
    let characters = sqlx::query!(
        "SELECT character_id, name, membership_reminder
        FROM planetside_characters
        WHERE user_id = (
            SELECT character_id
            FROM users
            WHERE discord_id = $1
        )",
        discord_id
    )
    .fetch_all(pool)
    .await?;

    let mut character_vec = Vec::new();

    for char in &characters {
        #[allow(clippy::cast_sign_loss)]
        let char_id = char.character_id as u64;
        character_vec.push(Character {
            character_id: char_id,
            name: CharacterName {
                first: char.name.clone(),
                first_lower: char.name.to_lowercase(),
            },
            membership_reminder: Some(MembershipReminderStatus {
                enabled: char.membership_reminder,
                last_reminder: None,
            }),
            ..Default::default()
        });
    }

    Ok(character_vec)
}

pub async fn reset_reminder_for_discord_users(
    pool: &PgPool,
    discord_ids: Vec<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE planetside_characters
        SET last_membership_reminder = NOW()
        WHERE user_id = (
            SELECT user_id
            FROM users
            WHERE discord_id = ANY($1)
        )",
        &discord_ids
    )
    .execute(pool)
    .await?;

    Ok(())
}
