use sqlx::{PgPool, Pool, Postgres};
use dotenvy::dotenv;
use std::env;
use crate::utils::logger::success;

pub type DbPool = Pool<Postgres>;

pub async fn init_db() -> Result<DbPool, sqlx::Error> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
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
            program_id INT,
            map_ids INT[],
            pinned_path TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            CONSTRAINT unique_title_version UNIQUE (title, version)
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION set_updated_at()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DROP TRIGGER IF EXISTS set_updated_at_trigger ON ebpf_programs;
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TRIGGER set_updated_at_trigger
        BEFORE UPDATE ON ebpf_programs
        FOR EACH ROW
        EXECUTE FUNCTION set_updated_at();
        "#
    )
    .execute(pool)
    .await?;

    success("Database migration successful!");
    Ok(())
}
