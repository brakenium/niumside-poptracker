use sqlx::{postgres::PgPoolOptions, Pool, Postgres, error::Error};

#[tokio::main]
pub async fn create(connection_string: &str) -> Result<Pool<Postgres>, Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string).await
}
