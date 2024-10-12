use crate::controllers::Languages;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Zone {
    pub id: i32,
    pub name: Option<Languages>,
    pub description: Option<Languages>,
}

/// Check if a zone exists in the database
///
/// # Arguments
///
/// * `db_pool` - The database pool to use
/// * `zones` - The zone ID to check
///
/// # Returns
///
/// * `Ok(bool)` - True if the zone exists, false otherwise
/// * `Err(sqlx::Error)` - The error returned by sqlx
#[allow(dead_code)]
pub async fn exists(db_pool: &PgPool, zones: &i32) -> Result<bool, sqlx::Error> {
    match sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM zone WHERE zone_id = $1)",
        zones
    )
        .fetch_one(db_pool)
        .await
    {
        Ok(result) => Ok(result.exists.unwrap_or(false)),
        Err(e) => Err(e),
    }
}

/// Get all zones from the database
///
/// # Arguments
///
/// * `db_pool` - The database pool to use
///
/// # Returns
///
/// * `Ok(Vec<(i32, String)>)` - A vector of tuples containing the zone ID and the zone name
/// * `Err(sqlx::Error)` - The error returned by sqlx
pub async fn get_all(db_pool: &PgPool) -> Result<Vec<Zone>, sqlx::Error> {
    sqlx::query!("SELECT zone_id, name, description FROM zone")
        .fetch_all(db_pool)
        .await
        .map(|zones| {
            //     take the zones record and return a vector of tuples containing the zone ID and the zone name
            zones.into_iter().map(|z| Zone {
                id: z.zone_id,
                name: Some(Languages {
                    en: z.name,
                }),
                description: Some(Languages {
                    en: z.description,
                }),
            }).collect()
        })
}

/// Get all zones from the database that exist
///
/// # Arguments
///
/// * `db_pool` - The database pool to use
/// * `zones` - The zone IDs to check
///
/// # Returns
///
/// * `Ok(Vec<(i32, String)>)` - A vector of tuples containing the zone ID and the zone name of the zones that exist
/// * `Err(sqlx::Error)` - The error returned by sqlx
#[allow(dead_code)]
pub async fn get_all_existing(
    db_pool: &PgPool,
    zones: &[i32],
) -> Result<Vec<(i32, Option<String>)>, sqlx::Error> {
    sqlx::query!(
        "SELECT zone_id, name FROM zone WHERE zone_id = ANY($1)",
        zones
    )
        .fetch_all(db_pool)
        .await
        .map(|zones| {
            // take the zones record and return a vector of tuples containing the zone ID and the zone name
            zones.into_iter().map(|z| (z.zone_id, z.name)).collect()
        })
}
