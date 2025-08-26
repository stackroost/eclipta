use crate::utils::db::init_db;
use crate::db::programs::delete_program;
use std::fs;
use clap::Parser;

#[derive(Parser)]
pub struct RemoveOptions {
    pub id: i32,
}

pub async fn handle_remove(opts: RemoveOptions) -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_db().await?;
    let program = sqlx::query!(
        "SELECT path, title FROM ebpf_programs WHERE id = $1",
        opts.id
    )
    .fetch_one(&pool)
    .await?;
    if fs::remove_file(&program.path).is_ok() {
        println!("[OK] Deleted file: {}", &program.path);
    } else {
        println!("[WARN] File not found or cannot delete: {}", &program.path);
    }
    delete_program(&pool, opts.id).await?;
    println!("[OK] Removed program '{}' (ID: {}) from database.", program.title, opts.id);

    Ok(())
}
