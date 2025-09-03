use crate::db::programs::insert_program;
use crate::utils::db::ensure_db_ready;
use crate::utils::logger::{success, error, info, warn};
use object::{Object, ObjectSection};
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

fn validate_elf_sections(path: &Path) -> Result<Vec<String>, String> {
    let data = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let file = object::File::parse(&*data).map_err(|e| format!("Failed to parse ELF: {}", e))?;

    let mut recognized: Vec<String> = Vec::new();
    for section in file.sections() {
        if let Ok(name) = section.name() {
            let name = name.trim();
            // Common eBPF program section name patterns
            let is_known =
                name == "xdp" || name.starts_with("xdp/") ||
                name == "tc" || name == "tc_ingress" || name == "tc_egress" ||
                name.starts_with("classifier/") || name.starts_with("cls/") ||
                name == "socket_filter" || name.starts_with("socket/") ||
                name.starts_with("kprobe/") || name.starts_with("kretprobe/") ||
                name.starts_with("tracepoint/") || name.starts_with("raw_tracepoint/") ||
                name.starts_with("uprobe/") || name.starts_with("uretprobe/") ||
                name.starts_with("lsm/") ||
                name.starts_with("cgroup/") || name.starts_with("cgroup_skb/") ||
                name.starts_with("cgroup_sock/") || name.starts_with("cgroup_sock_addr/") ||
                name.starts_with("cgroup_sockopt/") || name.starts_with("cgroup_sysctl/") ||
                name.starts_with("perf_event/") || name.starts_with("sk_msg/") ||
                name.starts_with("sk_skb/") || name.starts_with("sk_lookup/") ||
                name.starts_with("fentry/") || name.starts_with("fexit/");

            if is_known {
                recognized.push(name.to_string());
            }
        }
    }

    if recognized.is_empty() {
        return Err("No recognized eBPF program sections found".to_string());
    }

    Ok(recognized)
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

    // Step 2: Advanced ELF-level validation (section introspection)
    match validate_elf_sections(&opts.program) {
        Ok(sections) => {
            info(&format!("Found eBPF sections: {}", sections.join(", ")));
        }
        Err(e) => {
            error(&format!("Validation failed: {}", e));
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
    let pool = match ensure_db_ready().await {
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
    ).await {
        Ok(_) => success("Upload complete! Metadata stored in database."),
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.constraint() == Some("unique_title_version") {
                error("A program with this title and version already exists.");
                return Ok(());
            } else {
                error(&format!("Database insert failed: {}", db_err));
                return Ok(());
            }
        }
        Err(e) => {
            error(&format!("Database insert failed: {}", e));
            return Ok(());
        }
    }

    Ok(())
}

