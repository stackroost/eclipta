use clap::Args;
use std::{collections::HashMap, fs, path::PathBuf};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Args)]
pub struct ConfigOptions {
    #[arg(long)]
    pub agent: String,

    #[arg(long)]
    pub get: Option<String>,

    #[arg(long)]
    pub set: Option<String>,

    #[arg(long)]
    pub list: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AgentSettings {
    pub settings: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct AgentStatus {
    id: String,
    hostname: String,
    kernel: String,
    version: String,
    last_seen: String,
    uptime_secs: u64,
    cpu_load: Option<[f32; 3]>,
    mem_used_mb: Option<u64>,
    mem_total_mb: Option<u64>,
    process_count: Option<u64>,
    disk_used_mb: Option<u64>,
    disk_total_mb: Option<u64>,
    net_rx_kb: Option<u64>,
    net_tx_kb: Option<u64>,
    tcp_connections: Option<u64>,
    alert: Option<bool>,
}

pub async fn handle_config(opts: ConfigOptions) -> Result<()> {
    let runtime_path = PathBuf::from(format!("/run/eclipta/{}.json", opts.agent));
    let config_path = PathBuf::from(format!("/etc/eclipta/agent-{}.conf.json", opts.agent));

    // Load runtime status from /run/eclipta/<agent>.json
    let status_data = fs::read_to_string(&runtime_path)?;
    let agent: AgentStatus = serde_json::from_str(&status_data)?;

    println!("Agent Configuration for '{}':\n", agent.id);
    println!("Hostname      : {}", agent.hostname);
    println!("Kernel        : {}", agent.kernel);
    println!("Version       : {}", agent.version);
    println!("Uptime (secs) : {}", agent.uptime_secs);
    println!("Last Seen     : {}", agent.last_seen);

    if let Some(cpu) = agent.cpu_load {
        println!("CPU Load      : {:.1} / {:.1} / {:.1}", cpu[0], cpu[1], cpu[2]);
    }

    if let (Some(used), Some(total)) = (agent.mem_used_mb, agent.mem_total_mb) {
        println!("Memory        : {} MB / {} MB", used, total);
    }

    if let Some(proc) = agent.process_count {
        println!("Processes     : {}", proc);
    }

    if let (Some(du), Some(dt)) = (agent.disk_used_mb, agent.disk_total_mb) {
        println!("Disk Usage    : {} MB / {} MB", du, dt);
    }

    if let (Some(rx), Some(tx)) = (agent.net_rx_kb, agent.net_tx_kb) {
        println!("Network RX/TX : {} KB / {} KB", rx, tx);
    }

    if let Some(tc) = agent.tcp_connections {
        println!("TCP Conns     : {}", tc);
    }

    if let Some(alert) = agent.alert {
        println!("Alert         : {}", if alert { "YES" } else { "No" });
    }

    // Load config from /etc/eclipta/agent-xxx.conf.json
    let mut config: AgentSettings = if config_path.exists() {
        let data = fs::read_to_string(&config_path)?;
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AgentSettings::default()
    };

    if opts.list {
        println!("\nAgent Saved Settings:");
        for (key, value) in &config.settings {
            println!("{} = {}", key, value);
        }
    }

    if let Some(key) = opts.get {
        if let Some(val) = config.settings.get(&key) {
            println!("\n{} = {}", key, val);
        } else {
            println!("\nKey '{}' not found", key);
        }
    }

    if let Some(kv) = opts.set {
        let parts: Vec<&str> = kv.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid format. Use --set key=value"));
        }

        config.settings.insert(parts[0].to_string(), parts[1].to_string());
        fs::create_dir_all("/etc/eclipta")?;
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
        println!("\nUpdated config: {} = {}", parts[0], parts[1]);
    }

    Ok(())
}
