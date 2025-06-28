mod commands;
mod utils;

use crate::commands::logs::LogOptions;
use clap::{ Parser, Subcommand };
use commands::agent_logs::{ handle_agent_logs, AgentLogsOptions };
use commands::agents::{ handle_agents, AgentOptions };
use commands::agents_inspect::{ handle_inspect_agent, InspectAgentOptions };
use commands::daemon::handle_daemon;
use commands::inspect::{ handle_inspect, InspectOptions };
use commands::live::handle_live;
use commands::monitor::handle_monitor;
use commands::ping_all;
use commands::alerts::handle_alerts;
use commands::restart_agent::{ handle_restart_agent, RestartAgentOptions };
use commands::config::{ handle_config, ConfigOptions };
use commands::watch_cpu::{ handle_watch_cpu, WatchCpuOptions };
use commands::kill_agent::{handle_kill_agent, KillAgentOptions};
use commands::{
    load::handle_load,
    logs::handle_logs,
    status::run_status,
    unload::{ handle_unload, UnloadOptions },
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
    InspectAgent(InspectAgentOptions),
    RestartAgent(RestartAgentOptions),
    Agents(AgentOptions),
    AgentLogs(AgentLogsOptions),
    Live,
    Daemon,
    Monitor,
    PingAll,
    WatchCpu(WatchCpuOptions),
    Config(ConfigOptions),
    Alerts,
    KillAgent(KillAgentOptions),
}
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Welcome => run_welcome(),
        Commands::Status => run_status(),
        Commands::Load(opts) => handle_load(opts),
        Commands::Unload(opts) => handle_unload(opts),
        Commands::Inspect(opts) => handle_inspect(opts),
        Commands::Logs(opts) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_logs(opts));
        }
        Commands::Agents(opts) => handle_agents(opts),
        Commands::AgentLogs(opts) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_agent_logs(opts));
        }
        Commands::InspectAgent(opts) => handle_inspect_agent(opts),
        Commands::RestartAgent(opts) => handle_restart_agent(opts),
        Commands::Live => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_live());
        }
        Commands::Daemon => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_daemon());
        }
        Commands::Monitor => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_monitor()).unwrap();
        }
        Commands::PingAll => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(ping_all::handle_ping_all());
        }
        Commands::WatchCpu(opts) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_watch_cpu(opts)).unwrap();
        }
        Commands::Config(opts) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_config(opts)).unwrap();
        }
        Commands::Alerts => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(handle_alerts()).unwrap();
        }
        Commands::KillAgent(opts) => handle_kill_agent(opts).unwrap(),
        
    }
}
