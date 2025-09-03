use sqlx::{PgPool, Pool, Postgres};
use dotenvy::dotenv;
use std::env;
use crate::utils::logger::error;
use crate::db::migrations::{run_migrations, check_migration_status};

pub type DbPool = Pool<Postgres>;

pub async fn ensure_db_ready() -> Result<DbPool, Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPool::connect(&db_url).await?;
    
    // Check if migrations are needed
    let is_ready = check_migration_status(&pool).await?;
    
    if !is_ready {
        error("Database is not ready. Please run 'cargo run migrate' first to initialize the database.");
        return Err("Database not initialized".into());
    }
    
    Ok(pool)
}

pub async fn run_migrations_only() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPool::connect(&db_url).await?;
    
    run_migrations(&pool).await?;
    
    Ok(())
}
