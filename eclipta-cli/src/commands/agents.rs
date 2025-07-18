use clap::Args;
use std::{fs, path::PathBuf};
use serde::{Deserialize, Serialize};
use crate::utils::logger::{info, success, error};

#[derive(Args, Debug)]
pub struct AgentOptions {
    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub verbose: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgentStatus {
    id: String,
    hostname: String,
    kernel: String,
    version: String,
    last_seen: String,
    uptime_secs: u64,
}

pub fn handle_agents(opts: AgentOptions) {
    let dir = PathBuf::from("/run/eclipta");

    let files = match fs::read_dir(&dir) {
        Ok(f) => f,
        Err(e) => {
            error(&format!("❌ Failed to read /run/eclipta: {}", e));
            return;
        }
    };

    let mut agents = Vec::new();

    for entry in files.flatten() {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(agent) = serde_json::from_str::<AgentStatus>(&content) {
                agents.push(agent);
            }
        }
    }

    if agents.is_empty() {
        info("No agents found.");
        return;
    }

    if opts.json {
        let out = serde_json::to_string_pretty(&agents).unwrap();
        println!("{}", out);
        return;
    }

    success("🟢 Connected Agents:");
    for agent in &agents {
        println!(
            "• [{}] {} - {} (uptime: {}s)",
            agent.status_string(),
            agent.id,
            agent.hostname,
            agent.uptime_secs
        );
        if opts.verbose {
            println!("   └─ Kernel: {} | Version: {}", agent.kernel, agent.version);
        }
    }
}

impl AgentStatus {
    fn status_string(&self) -> &'static str {
        "online"
    }
}
