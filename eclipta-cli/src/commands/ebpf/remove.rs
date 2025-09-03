use crate::utils::db::ensure_db_ready;
use crate::db::programs::delete_program;
use std::fs;
use clap::Parser;
use sqlx::Row;

#[derive(Parser)]
pub struct RemoveOptions {
    pub id: i32,
}

pub async fn handle_remove(opts: RemoveOptions) -> Result<(), Box<dyn std::error::Error>> {
    // connect to DB
    let pool = match ensure_db_ready().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {e}");
            return Ok(()); // return gracefully instead of crashing
        }
    };

    // fetch program info (runtime query to avoid compile-time DB checks)
    let program_row = match sqlx::query(
        "SELECT path, title FROM ebpf_programs WHERE id = $1",
    )
    .bind(opts.id)
    .fetch_one(&pool)
    .await
    {
        Ok(row) => row,
        Err(e) => {
            eprintln!("❌ Could not find program with ID {}: {e}", opts.id);
            return Ok(());
        }
    };

    // try removing file
    let path: String = program_row.get("path");
    let title: String = program_row.get("title");

    match fs::remove_file(&path) {
        Ok(_) => println!("✅ Deleted file: {}", &path),
        Err(_) => println!("⚠️ File not found or cannot delete: {}", &path),
    }

    // delete from database
    match delete_program(&pool, opts.id).await {
        Ok(_) => println!("✅ Removed program '{}' (ID: {}) from database.", title, opts.id),
        Err(e) => eprintln!("❌ Failed to remove program from database: {e}"),
    }

    Ok(())
}
