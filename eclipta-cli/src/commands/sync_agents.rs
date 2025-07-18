use std::{fs, path::PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use clap::Args;
use anyhow::Result;

#[derive(Args)]
pub struct SyncAgentsOptions {}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentSnapshot {
    pub id: String,
    pub hostname: String,
    pub kernel: String,
    pub version: String,
    pub last_seen: DateTime<Utc>,
    pub uptime_secs: u64,
    pub cpu_load: [f32; 3],
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub process_count: u64,
    pub disk_used_mb: u64,
    pub disk_total_mb: u64,
    pub net_rx_kb: u64,
    pub net_tx_kb: u64,
    pub tcp_connections: u64,
    pub alert: bool
}

pub async fn handle_sync_agents(_: SyncAgentsOptions) -> Result<()> {
    let mut snapshots = Vec::new();
    let dir = PathBuf::from("/run/eclipta");

    if dir.exists() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let raw = fs::read_to_string(&path)?;
                match serde_json::from_str::<AgentSnapshot>(&raw) {
                    Ok(snapshot) => snapshots.push(snapshot),
                    Err(e) => eprintln!("⚠️ Failed to parse {}: {}", path.display(), e),
                }
            }
        }
    }

    let out_path = PathBuf::from("/etc/eclipta/agents/snapshot.json");
    fs::create_dir_all(out_path.parent().unwrap())?;
    fs::write(&out_path, serde_json::to_string_pretty(&snapshots)?)?;

    println!("✅ Synced {} agents to {}", snapshots.len(), out_path.display());

    Ok(())
}
