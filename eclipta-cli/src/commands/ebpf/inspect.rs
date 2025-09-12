use aya::Ebpf;
use clap::Args;
use std::path::PathBuf;
use crate::utils::logger::success;
use crate::utils::db::ensure_db_ready;
use crate::db::programs::{get_program_by_id, get_program_by_title};
use serde_json;
use anyhow::{Result, anyhow};

#[derive(Args, Debug)]
pub struct InspectOptions {
    /// Path to eBPF ELF file (alternative to --id/--title)
    #[arg(short, long, conflicts_with_all = ["id", "title"])]
    pub program: Option<PathBuf>,

    /// Program ID from database (alternative to --program/--title)
    #[arg(long, conflicts_with_all = ["program", "title"])]
    pub id: Option<i32>,

    /// Program title from database (alternative to --program/--id)
    #[arg(long, conflicts_with_all = ["program", "id"])]
    pub title: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,

    /// Show verbose technical details
    #[arg(long)]
    pub verbose: bool,
}

pub async fn handle_inspect(opts: InspectOptions) -> Result<()> {
    if opts.program.is_none() && opts.id.is_none() && opts.title.is_none() {
        return Err(anyhow!(
            "Please specify a program to inspect using one of:\n\
            --program <path>  (for file path)\n\
            --id <id>         (for database ID)\n\
            --title <title>   (for database title)\n\
            \n\
            Use --help for more information."
        ));
    }

    let (program_path, program_metadata) = if let Some(id) = opts.id {
        println!("Looking up program with ID {} in database...", id);
        let pool = ensure_db_ready().await
            .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;
        
        let program = get_program_by_id(&pool, id).await
            .map_err(|e| anyhow!("Database query failed: {}", e))?
            .ok_or_else(|| anyhow!("No program found with ID {}", id))?;
        
        println!("Found program: '{}' (v{})", program.title, program.version);
        
        let path = PathBuf::from(&program.path);
        if !path.exists() {
            return Err(anyhow!("Program file not found at: {}\nThe program may have been moved or deleted.", program.path));
        }
        
        (path, Some(program))
    } else if let Some(ref title) = opts.title {
        println!("Looking up program with title '{}' in database...", title);
        let pool = ensure_db_ready().await
            .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;
        
        let programs = get_program_by_title(&pool, title).await
            .map_err(|e| anyhow!("Database query failed: {}", e))?;
        
        match programs.len() {
            1 => {
                let program = programs[0].clone();
                println!("Found program: '{}' (v{})", program.title, program.version);
                
                let path = PathBuf::from(&program.path);
                if !path.exists() {
                    return Err(anyhow!("Program file not found at: {}\nThe program may have been moved or deleted.", program.path));
                }
                (path, Some(program))
            }
            n if n > 1 => {
                println!("Multiple programs found with title '{}':", title);
                for (i, prog) in programs.iter().enumerate() {
                    println!("  {}. ID: {}, Version: {}, Status: {}", i + 1, prog.id, prog.version, prog.status);
                }
                return Err(anyhow!("Multiple programs found with title '{}'. Please use --id to specify which one to inspect.", title));
            }
            _ => {
                return Err(anyhow!("No program found with title '{}'", title));
            }
        }
    } else if let Some(program_path) = opts.program {
        println!("Inspecting program file: {}", program_path.display());
        if !program_path.exists() {
            return Err(anyhow!("Program file not found at: {}", program_path.display()));
        }
        (program_path, None)
    } else {
        return Err(anyhow!("No program specified. Use --help for available options."));
    };

    println!("Parsing eBPF ELF file...");
    let bpf = match Ebpf::load_file(&program_path) {
        Ok(b) => {
            println!("Successfully parsed ELF file");
            b
        }
        Err(e) => {
            return Err(anyhow!("Failed to parse ELF file: {}\nMake sure the file is a valid compiled eBPF program.", e));
        }
    };

    let programs: Vec<_> = bpf.programs().map(|(name, _)| name.to_string()).collect();
    let maps: Vec<_> = bpf.maps().map(|(name, _)| name.to_string()).collect();
    let mut output_data = serde_json::json!({
        "elf_path": program_path.display().to_string(),
        "programs": programs,
        "maps": maps,
    });
    if let Some(ref metadata) = program_metadata {
        output_data["metadata"] = serde_json::json!({
            "id": metadata.id,
            "title": metadata.title,
            "version": metadata.version,
            "status": metadata.status,
            "path": metadata.path
        });
    }

    if opts.json {
        println!("{}", serde_json::to_string_pretty(&output_data).unwrap());
    } else {
        success(&format!("Inspecting eBPF Program: {}", program_path.display()));
        
        if let Some(ref metadata) = program_metadata {
            println!("\nProgram Metadata:");
            println!("  ID: {}", metadata.id);
            println!("  Title: {}", metadata.title);
            println!("  Version: {}", metadata.version);
            println!("  Status: {}", metadata.status);
            println!("  Path: {}", metadata.path);
        }
        
        println!("\neBPF Programs:");
        if programs.is_empty() {
            println!("  (no programs found)");
        } else {
            for name in &programs { 
                println!("  {}", name); 
            }
        }
        
        println!("\neBPF Maps:");
        if maps.is_empty() {
            println!("  (no maps found)");
        } else {
            for name in &maps { 
                println!("  {}", name); 
            }
        }
        
        if opts.verbose {
            println!("\nTechnical Details:");
            println!("  ELF Path: {}", program_path.display());
            println!("  Programs Count: {}", programs.len());
            println!("  Maps Count: {}", maps.len());
        }
    }

    Ok(())
}
