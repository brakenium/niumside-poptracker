use crate::census::constants::{CharacterID, Faction, ZoneID};
use crate::census::rest::client::{CensusRequestableObject, CensusRestClient};
use crate::census::structs::character::{Character, CharacterName};
use futures::StreamExt;
use sqlx::PgPool;
use tracing::{error, info};
use crate::census::CENSUS_URL;

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

async fn update_from_source(db_pool: &PgPool) {
    let zones = tokio::spawn(update_zones(db_pool));
    let worlds = tokio::spawn(update_worlds(db_pool));

    let (zones, worlds) = tokio::join!(zones, worlds);

    if let Err(e) = zones {
        error!("Zone update task failed: {e}");
    }

    if let Err(e) = worlds {
        error!("World update task failed: {e}");
    }
}

async fn fetch_zones() -> Result<Vec<CensusZoneResponse>, reqwest::Error> {
    let request_url = CENSUS_URL
        .join("/get/ps2/zone?c:lang=en&c:show=zone_id,name,description&c:limit=99999999")
        .expect("Invalid zone request URL");

    let response = reqwest::get(request_url)
        .await?
        .json::<ZoneResponse>()
        .await?;

    Ok(response.zone_list)
}

pub async fn update_zones(db_pool: &PgPool) {
    let zones = match fetch_zones().await {
        Ok(zones) => zones,
        Err(err) => {
            error!("Unable to fetch zones: {err}");
            return;
        }
    };

    info!("Got {} zones", zones.len());

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
            i32::try_from(zone.zone_id.0).ok(),
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
                    character_id: match CharacterID::try_from(character.character_id) {
                        Ok(char_id) => char_id,
                        Err(err) => {
                            error!("unable to convert character id from DB into CharacterID datatype: {err}");
                            continue;
                        }
                    },
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
        let char_id = match i64::try_from(character.character_id) {
            Ok(char_id) => char_id,
            Err(err) => {
                error!("Unable to convert character_id ({}) into Character id: {err}", character.character_id);
                continue;
            }
        };

        match sqlx::query!(
            "UPDATE planetside_characters
            SET
                name = $2,
                faction_id = $3
            WHERE character_id = $1",
            char_id,
            character.name.first,
            i16::try_from(character.faction).ok(),
        )
        .execute(db_pool)
        .await {
            Ok(_) => {}
            Err(err) => error!("Error while updating character in database: {err}"),
        }
    }
}

pub async fn run(db_pool: &PgPool, census_rest_client: &CensusRestClient) {
    loop {
        update_characters(db_pool, census_rest_client).await;
        update_from_source(db_pool).await;
        tokio::time::sleep(tokio::time::Duration::from_hours(1)).await;
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
