mod commands;
mod utils;
mod db;

use crate::commands::logs::LogOptions;
use clap::{Parser, Subcommand};
use commands::alerts::handle_alerts;
use commands::check_db::{handle_check_db, CheckDbOptions};
use commands::config::{handle_config, ConfigOptions};
use commands::daemon::handle_daemon;
use commands::inspect::{handle_inspect, InspectOptions};
use commands::monitor::handle_monitor;
use commands::ping_all;
use commands::upload::{handle_upload, UploadOptions};
use commands::version::{handle_version, VersionOptions};
use commands::watch_cpu::{handle_watch_cpu, WatchCpuOptions};
use commands::{
    load::handle_load,
    logs::handle_logs,
    status::run_status,
    unload::{handle_unload, UnloadOptions},
    welcome::run_welcome,
};

#[derive(Parser)]
#[command(name = "eclipta")]
#[command(about = "eclipta CLI - self-hosted observability platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Welcome,
    Status,
    Load(commands::load::LoadOptions),
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
    Run(commands::run::RunOptions),
    CheckDb(CheckDbOptions),
    Upload(UploadOptions),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Welcome => run_welcome(),
        Commands::Status => run_status(),
        Commands::Load(opts) => handle_load(opts),
        Commands::Unload(opts) => handle_unload(opts),
        Commands::Inspect(opts) => handle_inspect(opts),
        Commands::Logs(opts) => handle_logs(opts).await,
        Commands::Daemon => handle_daemon().await,
        Commands::Monitor => handle_monitor().await.unwrap(),
        Commands::PingAll => ping_all::handle_ping_all().await,
        Commands::WatchCpu(opts) => handle_watch_cpu(opts).await.unwrap(),
        Commands::Config(opts) => handle_config(opts).await.unwrap(),
        Commands::Alerts => handle_alerts().await.unwrap(),
        Commands::Version(opts) => handle_version(opts).await.unwrap(),
        Commands::Run(opts) => commands::run::handle_run(opts).await,
        Commands::CheckDb(opts) => handle_check_db(opts).await.unwrap(),
        Commands::Upload(opts) => {
            if let Err(e) = handle_upload(opts).await {
                eprintln!("Upload failed: {}", e);
            }
        }
    }
}
