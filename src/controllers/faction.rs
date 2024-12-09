use sqlx::PgPool;

/// Check if a faction exists in the database
///
/// # Arguments
///
/// * `db_pool` - The database pool to use
/// * `factions` - The faction ID to check
///
/// # Returns
///
/// * `Ok(bool)` - True if the faction exists, false otherwise
/// * `Err(sqlx::Error)` - The error returned by sqlx
#[allow(dead_code)]
pub async fn exists(db_pool: &PgPool, factions: &i16) -> Result<bool, sqlx::Error> {
    match sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM faction WHERE faction_id = $1)",
        factions
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(result) => Ok(result.exists.unwrap_or(false)),
        Err(e) => Err(e),
    }
}

/// Get all factions from the database
///
/// # Arguments
///
/// * `db_pool` - The database pool to use
///
/// # Returns
///
/// * `Ok(Vec<(i32, String)>)` - A vector of tuples containing the faction ID and the faction name
/// * `Err(sqlx::Error)` - The error returned by sqlx
#[allow(dead_code)]
pub async fn get_all(db_pool: &PgPool) -> Result<Vec<(i16, Option<String>)>, sqlx::Error> {
    sqlx::query!("SELECT faction_id, name FROM faction")
        .fetch_all(db_pool)
        .await
        .map(|factions| {
            //     take the factions record and return a vector of tuples containing the faction ID and the faction name
            factions
                .into_iter()
                .map(|f| (f.faction_id, f.name))
                .collect()
        })
}

/// Get all factions from the database that exist
///
/// # Arguments
///
/// * `db_pool` - The database pool to use
/// * `factions` - The faction IDs to check
///
/// # Returns
///
/// * `Ok(Vec<(i32, String)>)` - A vector of tuples containing the faction ID and the faction name of the factions that exist
/// * `Err(sqlx::Error)` - The error returned by sqlx
#[allow(dead_code)]
pub async fn get_all_existing(
    db_pool: &PgPool,
    factions: &[i16],
) -> Result<Vec<(i16, Option<String>)>, sqlx::Error> {
    sqlx::query!(
        "SELECT faction_id, name FROM faction WHERE faction_id = ANY($1)",
        factions
    )
    .fetch_all(db_pool)
    .await
    .map(|factions| {
        // take the factions record and return a vector of tuples containing the faction ID and the faction name
        factions
            .into_iter()
            .map(|f| (f.faction_id, f.name))
            .collect()
    })
}
