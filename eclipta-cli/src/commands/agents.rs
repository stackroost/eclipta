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
            error(&format!("Failed to read /run/eclipta: {}", e));
            return;
        }
    };

    let mut agents = Vec::new();

    for entry in files.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&path) {
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

    success("Connected Agents:");
    for agent in &agents {
        let status = agent.status_string();
        let badge = match status {
            "online" => "🟢",
            "offline" => "🔴",
            _ => "❓",
        };

        println!(
            "• [{} {}] {} - {} (uptime: {}s)",
            badge,
            status,
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
        let pid_path = PathBuf::from(format!("/run/eclipta/{}.pid", self.id));

        let pid = match fs::read_to_string(&pid_path) {
            Ok(content) => match content.trim().parse::<i32>() {
                Ok(n) => n,
                Err(_) => return "offline",
            },
            Err(_) => return "offline",
        };

        let proc_path = PathBuf::from(format!("/proc/{}", pid));
        if proc_path.exists() {
            "online"
        } else {
            "offline"
        }
    }
}
