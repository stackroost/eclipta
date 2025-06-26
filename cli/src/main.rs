mod commands;
mod utils;

use clap::{Parser, Subcommand};
use commands::{welcome::run_welcome, status::run_status, load::handle_load, logs::handle_logs, unload::{handle_unload, UnloadOptions}};
use crate::commands::logs::LogOptions;
use commands::inspect::{handle_inspect, InspectOptions};
use commands::agents::{handle_agents, AgentOptions};




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
    Agents(AgentOptions),

}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Welcome => run_welcome(),
        Commands::Status => run_status(),
        Commands::Load(opts) => handle_load(opts),
        Commands::Unload(opts) => handle_unload(opts),
        Commands::Inspect(opts) => handle_inspect(opts),
        Commands::Agents(opts) => handle_agents(opts),
        Commands::Logs(opts) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_logs(opts));
        }
    }
}
