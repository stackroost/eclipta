mod commands;
mod utils;

use clap::{Parser, Subcommand};
use commands::{welcome::run_welcome, status::run_status};

#[derive(Parser)]
#[command(name = "eclipta")]
#[command(about = "eclipta CLI - self-hosted observability platform", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Welcome,
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Welcome => run_welcome(),
        Commands::Status => run_status(),
    }
}