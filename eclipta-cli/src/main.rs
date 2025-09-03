mod commands;
mod utils;
mod db;


use clap::{Parser, Subcommand};

// SYSTEM COMMANDS
use crate::commands::system::{
    logs::{handle_logs, LogOptions},
    monitor::handle_monitor,
    status::run_status,
    watch_cpu::{handle_watch_cpu, WatchCpuOptions},
};

// EBPf PROGRAM COMMANDS
use crate::commands::ebpf::{
    inspect::{handle_inspect, InspectOptions},
    load::handle_load,
    unload::{handle_unload, UnloadOptions},
    upload::{handle_upload, UploadOptions},
    list::handle_list,
    remove::{handle_remove, RemoveOptions},
};

// NETWORK COMMANDS
use crate::commands::network::{
    alerts::handle_alerts,
    ping_all::handle_ping_all,
};

// CONFIG COMMANDS
use crate::commands::config::{
    config::{handle_config, ConfigOptions},
    daemon::handle_daemon,
};

// STORE / DB COMMANDS
use crate::commands::store::check_db::{handle_check_db, CheckDbOptions};
use crate::commands::store::migrate::{handle_migrate, MigrateOptions};

// OTHER GLOBAL COMMANDS
use crate::commands::{
    run::{handle_run, RunOptions},
    version::{handle_version, VersionOptions},
    welcome::run_welcome,
};

#[derive(Parser)]
#[command(name = "eclipta")]
#[command(about = "Eclipta CLI - self-hosted observability platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Welcome,
    Status,
    Load(commands::ebpf::load::LoadOptions),
    Logs(LogOptions),
    Unload(UnloadOptions),
    Inspect(InspectOptions),
    Daemon,
    Monitor,
    PingAll,
    WatchCpu(WatchCpuOptions),
    Config(ConfigOptions),
    Alerts,
    Version(VersionOptions),
    Run(RunOptions),
    CheckDb(CheckDbOptions),
    Migrate(MigrateOptions),
    Upload(UploadOptions),
    List,
    Remove(RemoveOptions), 
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = handle_command(cli.command).await {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

async fn handle_command(cmd: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        Commands::Welcome => run_welcome(),
        Commands::Status => run_status(),
        Commands::Load(opts) => handle_load(opts).await?,
        Commands::Unload(opts) => handle_unload(opts),
        Commands::Inspect(opts) => handle_inspect(opts),
        Commands::Logs(opts) => handle_logs(opts).await,
        Commands::Daemon => handle_daemon().await,
        Commands::Monitor => handle_monitor().await?,
        Commands::PingAll => handle_ping_all().await,
        Commands::WatchCpu(opts) => handle_watch_cpu(opts).await?,
        Commands::Config(opts) => handle_config(opts).await?,
        Commands::Alerts => handle_alerts().await?,
        Commands::Version(opts) => handle_version(opts).await?,
        Commands::Run(opts) => handle_run(opts).await,
        Commands::CheckDb(opts) => handle_check_db(opts).await?,
        Commands::Migrate(opts) => handle_migrate(opts).await?,
        Commands::Upload(opts) => {
            if let Err(e) = handle_upload(opts).await {
                eprintln!("[UPLOAD ERROR] {}", e);
            }
        }
        Commands::List => {
            if let Err(e) = handle_list().await {
                eprintln!("[LIST ERROR] {}", e);
            }
        }
        Commands::Remove(opts) => {
            if let Err(e) = handle_remove(opts).await {
                eprintln!("[REMOVE ERROR] {}", e);
            }
        }
    }

    Ok(())
}
