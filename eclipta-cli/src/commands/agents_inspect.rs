use clap::Args;
use std::{fs, path::PathBuf};
use serde::{Deserialize, Serialize};
use crate::utils::logger::{error, success, info};

#[derive(Args, Debug)]
pub struct InspectAgentOptions {
    /// Agent ID (like agent-001)
    pub id: String,

    /// Output JSON
    #[arg(long)]
    pub json: bool,
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

pub fn handle_inspect_agent(opts: InspectAgentOptions) {
    let path = PathBuf::from(format!("/run/eclipta/{}.json", opts.id));

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            error(&format!("Failed to read agent file: {}", e));
            return;
        }
    };

    let agent: AgentStatus = match serde_json::from_str(&content) {
        Ok(a) => a,
        Err(e) => {
            error(&format!("Invalid JSON in agent file: {}", e));
            return;
        }
    };

    if opts.json {
        let output = serde_json::to_string_pretty(&agent).unwrap();
        println!("{}", output);
        return;
    }

    success(&format!("Agent: {}", agent.id));
    info(&format!("Hostname: {}", agent.hostname));
    info(&format!("Kernel: {}", agent.kernel));
    info(&format!("Uptime: {}s", agent.uptime_secs));
    info(&format!("Last Seen: {}", agent.last_seen));
    info(&format!("Version: {}", agent.version));
}
