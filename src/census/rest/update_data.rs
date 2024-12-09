use crate::census::constants::{Faction, ZoneID};
use crate::census::rest::client::{CensusRequestableObject, CensusRestClient};
use crate::census::structs::character::{Character, CharacterName};
use futures::StreamExt;
use sqlx::PgPool;
use tracing::{error, info};

const LITHAFALCON_BASE_URL: &str = "https://census.lithafalcon.cc";

#[derive(serde::Deserialize, Default)]
#[allow(dead_code)]
struct CensusMultiLanguage {
    de: Option<String>,
    en: Option<String>,
    es: Option<String>,
    fr: Option<String>,
    it: Option<String>,
    ko: Option<String>,
    pt: Option<String>,
    ru: Option<String>,
    tr: Option<String>,
    zh: Option<String>,
}

#[derive(serde::Deserialize)]
struct CensusZoneResponse {
    zone_id: ZoneID,
    name: Option<CensusMultiLanguage>,
    description: Option<CensusMultiLanguage>,
}

#[derive(serde::Deserialize)]
struct ZoneResponse {
    zone_list: Vec<CensusZoneResponse>,
}

pub async fn update_from_lithafalcon(db_pool: &PgPool) {
    let request_url = format!("{LITHAFALCON_BASE_URL}/get/PS2/zone?c:censusJSON=false&c:lang=en&c:show=zone_id,name,description");
    let request = match reqwest::get(request_url).await {
        Ok(response) => response,
        Err(e) => {
            error!("Error while requesting zones from lithafalcon: {e}");
            return;
        }
    }
    .json::<ZoneResponse>()
    .await;

    match request {
        Ok(response) => {
            let zones = response.zone_list;
            info!("Got {} zones from lithafalcon", zones.len());

            let mut transaction = match db_pool.begin().await {
                Ok(transaction) => transaction,
                Err(e) => {
                    error!("Error while starting transaction: {e}");
                    return;
                }
            };

            for zone in zones {
                let zone_name = zone.name.unwrap_or_else(CensusMultiLanguage::default);
                let zone_description = zone
                    .description
                    .unwrap_or_else(CensusMultiLanguage::default);

                #[allow(clippy::cast_possible_wrap)]
                match sqlx::query!(
                    "INSERT INTO zone
                    (zone_id, name, description)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (zone_id) DO UPDATE SET name = $2, description = $3",
                    zone.zone_id.0 as i32,
                    zone_name.en,
                    zone_description.en
                )
                .execute(&mut *transaction)
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error while inserting zone into database: {e}");
                    }
                }
            }

            match transaction.commit().await {
                Ok(()) => {}
                Err(e) => {
                    error!("Error while committing transaction: {e}");
                }
            }
        }
        Err(e) => {
            error!("Error while requesting zones from lithafalcon: {e}");
        }
    }
}

pub async fn update_characters(db_pool: &PgPool, census_rest_client: &CensusRestClient) {
    let mut characters = sqlx::query!(
        "SELECT character_id
        FROM planetside_characters"
    )
    .fetch(db_pool);

    while let Some(character) = characters.next().await {
        let character = match character {
            Ok(character) => {
                let mut char = Character {
                    #[allow(clippy::cast_sign_loss)]
                    character_id: character.character_id as u64,
                    name: CharacterName {
                        first: String::new(),
                        first_lower: String::new(),
                    },
                    membership_reminder: None,
                    times: None,
                    faction: Faction::Unknown,
                };

                if char.update_from_rest(census_rest_client).await.is_err() {
                    error!("Error while updating character from REST");
                    continue;
                }

                char
            }
            Err(e) => {
                error!("Error while fetching character from database: {e}");
                continue;
            }
        };

        #[allow(clippy::cast_possible_wrap)]
        let char_id = character.character_id as i64;

        let insert_action = sqlx::query!(
            "UPDATE planetside_characters
            SET
                name = $2,
                faction_id = $3
            WHERE character_id = $1",
            char_id,
            character.name.first,
            character.faction as i16,
        )
        .execute(db_pool)
        .await;

        if insert_action.is_err() {
            error!("Error while updating character in database");
        }
    }
}

pub async fn run(db_pool: &PgPool, census_rest_client: &CensusRestClient) {
    loop {
        update_characters(db_pool, census_rest_client).await;
        update_from_lithafalcon(db_pool).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60)).await;
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parsing_from_lithafalcon() {
        let request_url = format!("{LITHAFALCON_BASE_URL}/get/PS2/zone?c:censusJSON=false&c:lang=en&c:show=zone_id,name,description");
        match reqwest::get(request_url).await {
            Ok(response) => response,
            Err(e) => {
                panic!("Error while requesting zones from lithafalcon: {e}");
            }
        }
        .json::<ZoneResponse>()
        .await
        .expect("Unable to parse JSON");
    }
}
