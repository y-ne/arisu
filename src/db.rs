use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn connect() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to database")
}
