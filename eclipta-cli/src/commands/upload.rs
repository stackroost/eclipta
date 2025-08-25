use crate::db::programs::insert_program;
use crate::utils::db::init_db;
use crate::utils::logger::{success, error, info, warn};
use aya::Ebpf;
use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Args, Debug)]
pub struct UploadOptions {
    /// Path to eBPF ELF file (.o)
    #[arg(short, long)]
    pub program: PathBuf,

    /// Program title
    #[arg(long)]
    pub title: String,

    /// Program description
    #[arg(long)]
    pub description: String,

    /// Version (default: v1.0.0)
    #[arg(long, default_value = "v1.0.0")]
    pub version: String,
}

pub async fn handle_upload(opts: UploadOptions) -> Result<(), Box<dyn std::error::Error>> {
    info("Starting upload process...");

    // Step 1: Validate file exists
    if !opts.program.exists() {
        error("Provided eBPF program file does not exist.");
        return Ok(());
    }
    if !opts.program.is_file() {
        error("Provided path is not a valid file.");
        return Ok(());
    }

    // Step 2: Validate ELF with aya
    match Ebpf::load_file(&opts.program) {
        Ok(_) => info("ELF validation successful."),
        Err(e) => {
            error(&format!("Invalid eBPF program: {}", e));
            return Ok(());
        }
    }

    // Step 3: Copy program into storage directory with unique name
    let storage_dir = Path::new("/var/lib/eclipta/programs");
    if !storage_dir.exists() {
        warn("Storage directory missing. Creating...");
        fs::create_dir_all(storage_dir)?;
    }

    let unique_name = format!(
        "{}-{}.o",
        opts.title.replace(' ', "_"),
        Uuid::new_v4()
    );
    let dest_path = storage_dir.join(unique_name);

    match fs::copy(&opts.program, &dest_path) {
        Ok(_) => info(&format!("Program stored at {}", dest_path.display())),
        Err(e) => {
            error(&format!("Failed to copy program: {}", e));
            return Ok(());
        }
    }

    // Step 4: Insert metadata into Postgres
    let pool = match init_db().await {
        Ok(p) => p,
        Err(e) => {
            error(&format!("Failed to connect to database: {}", e));
            return Ok(());
        }
    };

    match insert_program(
        &pool,
        &opts.title,
        &opts.description,
        &opts.version,
        &dest_path.to_string_lossy(),
    )
    .await
    {
        Ok(_) => success("Upload complete! Metadata stored in database."),
        Err(e) => error(&format!("Database insert failed: {}", e)),
    }

    Ok(())
}
