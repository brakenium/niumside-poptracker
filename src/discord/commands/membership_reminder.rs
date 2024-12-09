use crate::census::rest::client::CensusRequestableObject;
use crate::census::structs::character::Character;
use crate::controllers;
use crate::discord::formatting::DEFAULT_EMBED_COLOR;
use crate::discord::icons::Icons;
use crate::discord::{Context, Error};
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;
use tracing::error;

/// Change daily login reminder settings. Will remind 1 hour before daily login reset.
#[poise::command(slash_command, track_edits, subcommands("specific", "all"))]
pub async fn dailyloginreminder(
    ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}

/// Adjust daily login reminder for specific characters
#[poise::command(slash_command, track_edits)]
pub async fn specific(
    ctx: Context<'_>,
    #[description = "Character to enable daily login reminder for"]
    characters: Vec<String>,
    #[description = "Enable or disable daily login reminder"]
    enable: bool,
) -> Result<(), Error> {
    ctx.defer().await?;
    let user = match controllers::user::insert_or_update(&ctx.data().db_pool, &ctx.author().id.get()).await {
        Ok(user) => user,
        Err(e) => {
            error!("Error while updating user in database: {e}");
            return Err(Error::from("Failed to update user in database"));
        }
    };

    let mut updated_characters: Vec<Character> = Vec::new();
    let mut failed_characters: Vec<String> = Vec::new();

    for char in characters {
        let character = match Character::get_by_name(&ctx.data().census_rest_client, &char).await {
            Ok(character) => character,
            Err(e) => {
                error!("Error while fetching character from REST: {e}");
                failed_characters.push(char);
                continue;
            }
        };
        match controllers::character::insert_or_update_character(
            &ctx.data().db_pool,
            &character,
            &user,
            &enable,
        ).await {
            Ok(()) => updated_characters.push(character),
            Err(e) => {
                error!("Error while updating character in database: {e}");
                failed_characters.push(char);
                continue;
            }
        };
    }

    let mut description = String::new();

    if updated_characters.is_empty() {
        description.push_str("No characters updated");
        ctx.say(description).await?;
        return Ok(());
    }

    if enable {
        description.push_str("Enabled daily login reminder for the following characters:\n");
    } else {
        description.push_str("Disabled daily login reminder for the following characters:\n");
    }

    for character in updated_characters {
        let wrapped_icons = Icons::try_from(character.faction)
            .unwrap_or(Icons::Ps2White)
            .to_discord_emoji();

        let icon: String = wrapped_icons.map_or_else(
            || character.faction.to_string(),
            |emoji| emoji.to_string(),
        );

        description.push_str(&format!("{} {}", icon, character.name.first));
    }

    if !failed_characters.is_empty() {
        description.push_str("\n\nFailed to update the following characters:\n");
        for character in failed_characters {
            description.push_str(&format!("- {character}"));
        }
    }

    let embed = CreateEmbed::default()
        .title("Daily Login Reminder")
        .description(description)
        .color(DEFAULT_EMBED_COLOR);

    let reply = CreateReply::default()
        .embed(embed);

    ctx.send(reply).await?;

    Ok(())
}

/// Adjust daily login reminder for all currently tracked characters
#[poise::command(slash_command, track_edits)]
pub async fn all(
    ctx: Context<'_>,
    #[description = "Enable or disable daily login reminder"]
    enable: bool,
) -> Result<(), Error> {
    ctx.defer().await?;
    sqlx::query!(
        "UPDATE planetside_characters
        SET membership_reminder = $1
        WHERE user_id = (
            SELECT user_id
            FROM users
            WHERE discord_id = $2
        )",
        enable,
        ctx.author().id.get() as i64,
    )
        .execute(&ctx.data().db_pool)
        .await?;

    ctx.say("Updated daily login reminder settings for all characters").await?;

    Ok(())
}
