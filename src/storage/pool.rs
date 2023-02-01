use sqlx::{postgres::PgPoolOptions, Pool, Postgres, error::Error};

pub async fn create(connection_string: &str) -> Result<Pool<Postgres>, Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string).await?;

    sqlx::migrate!().run(&pool.clone()).await?;
    Ok(pool)
}
