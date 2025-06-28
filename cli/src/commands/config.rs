use clap::Args;
use std::{fs, path::PathBuf};
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Args)]
pub struct ConfigOptions {
    #[arg(long)]
    pub agent: String,
}

#[derive(Debug, Deserialize)]
struct AgentConfig {
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
    let path = PathBuf::from(format!("/run/eclipta/{}.json", opts.agent));
    let data = fs::read_to_string(&path)?;
    let agent: AgentConfig = serde_json::from_str(&data)?;

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
        println!("Alert       : {}", if alert { "YES" } else { "No" });
    }

    Ok(())
}
