mod commands;
mod utils;

use clap::{Parser, Subcommand};
use commands::{load::handle_load, logs::LogOptions, status::run_status, welcome::run_welcome};
use commands::logs::handle_logs;

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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Welcome => run_welcome(),
        Commands::Status => run_status(),
        Commands::Load(opts) => handle_load(opts),
        Commands::Logs(opts) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_logs(opts));
        }
    }
}
