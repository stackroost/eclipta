use sqlx::{Pool, Postgres, Row};
use crate::utils::logger::{success, info};

pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    info("Checking database migrations...");
    
    // Create migrations table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;

    // Check if ebpf_programs table exists
    let table_exists: bool = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'ebpf_programs'
        )"
    )
    .fetch_one(pool)
    .await
    .map(|row| row.get::<bool, _>(0))?
    ;

    if !table_exists {
        info("Creating ebpf_programs table...");
        create_ebpf_programs_table(pool).await?;
        create_updated_at_trigger(pool).await?;
        
        // Record migration
        sqlx::query(
            "INSERT INTO migrations (name) VALUES ($1)"
        )
        .bind("001_create_ebpf_programs_table")
        .execute(pool)
        .await?;
        
        success("Database migration completed successfully!");
    } else {
        info("Database is up to date");
    }

    Ok(())
}

async fn create_ebpf_programs_table(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE ebpf_programs (
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

    Ok(())
}

async fn create_updated_at_trigger(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // Create the function
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

    // Create the trigger
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

    Ok(())
}

pub async fn check_migration_status(pool: &Pool<Postgres>) -> Result<bool, sqlx::Error> {
    let table_exists: bool = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'ebpf_programs'
        )"
    )
    .fetch_one(pool)
    .await
    .map(|row| row.get::<bool, _>(0))?;

    Ok(table_exists)
}
