use sqlx::{PgPool, Pool, Postgres};
use dotenvy::dotenv;
use std::env;
use crate::utils::logger::success;


pub type DbPool = Pool<Postgres>;

pub async fn init_db() -> Result<DbPool, sqlx::Error> {
    dotenv().ok(); 
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPool::connect(&db_url).await?;

    run_migrations(&pool).await?;
    Ok(pool)
}

async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ebpf_programs (
            id SERIAL PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            version TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'deactive',
            path TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;

    success("Database migration successful!");
    Ok(())
}