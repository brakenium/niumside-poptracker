use sqlx::PgPool;

/// Check if a world exists in the database
///
/// # Arguments
///
/// * `db_pool` - The database pool to use
/// * `worlds` - The world ID to check
///
/// # Returns
///
/// * `Ok(bool)` - True if the world exists, false if it does not
/// * `Err(sqlx::Error)` - The error returned by sqlx
#[allow(dead_code)]
pub async fn exists(db_pool: &PgPool, worlds: &i32) -> Result<bool, sqlx::Error> {
    match sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM world WHERE world_id = $1)",
        worlds
    )
    .fetch_one(db_pool)
    .await
    {
        Ok(result) => Ok(result.exists.unwrap_or(false)),
        Err(e) => Err(e),
    }
}

// Get all worlds from the database
//
// # Arguments
//
// * `db_pool` - The database pool to use
//
// # Returns
//
// * `Ok(Vec<(i32, String)>)` - A vector of tuples containing the world ID and the world name
// * `Err(sqlx::Error)` - The error returned by sqlx
pub async fn get_all(db_pool: &PgPool) -> Result<Vec<(i32, Option<String>)>, sqlx::Error> {
    sqlx::query!("SELECT world_id, name FROM world")
        .fetch_all(db_pool)
        .await
        .map(|worlds| {
            //     take the worlds record and return a vector of tuples containing the world ID and the world name
            worlds.into_iter().map(|w| (w.world_id, w.name)).collect()
        })
}

// Get all worlds from the database that exist
//
// # Arguments
//
// * `db_pool` - The database pool to use
// * `worlds` - The world IDs to check
//
// # Returns
//
// * `Ok(Vec<(i32, String)>)` - A vector of tuples containing the world ID and the world name
// * `Err(sqlx::Error)` - The error returned by sqlx
pub async fn get_all_existing(
    db_pool: &PgPool,
    worlds: &[i32],
) -> Result<Vec<(i32, Option<String>)>, sqlx::Error> {
    sqlx::query!(
        "SELECT world_id, name FROM world WHERE world_id = ANY($1)",
        worlds
    )
    .fetch_all(db_pool)
    .await
    .map(|worlds| {
        // take the worlds record and return a vector of tuples containing the world ID and the world name
        worlds.into_iter().map(|w| (w.world_id, w.name)).collect()
    })
}
