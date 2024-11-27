use sqlx::PgPool;
use tracing::{error, trace};

pub async fn insert_or_update(
    db_pool: &PgPool,
    discord_id: &u64,
) -> Result<i32, sqlx::Error> {
    let insert_action = sqlx::query!(
        "INSERT INTO users(
            discord_id
        ) VALUES ($1)
        ON CONFLICT (discord_id) DO NOTHING
        RETURNING user_id",
        *discord_id as i64
    )
        .fetch_one(db_pool)
        .await;

    match insert_action {
        Ok(user) => Ok(user.user_id),
        Err(e) => {
            let error_as_str = e.to_string();
            if error_as_str == *"no rows returned by a query that expected to return at least one row" {
                let user_id = get_by_discord_id(db_pool, discord_id).await?;
                Ok(user_id)
            } else {
                error!("Error while retrieving user from database: {:?}", e);
                Err(e)
            }
        }
    }
}

pub async fn get_by_discord_id(
    db_pool: &PgPool,
    discord_id: &u64,
) -> Result<i32, sqlx::Error> {
    let user = sqlx::query!(
        "SELECT user_id
        FROM users
        WHERE discord_id = $1",
        *discord_id as i64
    )
        .fetch_one(db_pool)
        .await;

    match user {
        Ok(user) => Ok(user.user_id),
        Err(e) => {
            error!("Error while fetching user from database: {:?}", e);
            Err(e)
        }
    }
}