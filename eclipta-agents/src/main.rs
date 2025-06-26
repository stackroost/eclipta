use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::{thread, time};
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
struct AgentStatus {
    id: String,
    hostname: String,
    kernel: String,
    version: String,
    last_seen: String,
    uptime_secs: u64,
}

fn main() {
    let interval = time::Duration::from_secs(5);
    let agent_id = "agent-001";

    loop {
        let hostname = get_hostname();
        let kernel = get_kernel();
        let version = "v0.1.0";
        let now = Utc::now().to_rfc3339();
        let uptime_secs = get_uptime_secs();

        let status = AgentStatus {
            id: agent_id.to_string(),
            hostname,
            kernel,
            version: version.to_string(),
            last_seen: now,
            uptime_secs,
        };

        let status_dir = PathBuf::from("/run/eclipta");
        fs::create_dir_all(&status_dir).ok();

        let json = serde_json::to_string_pretty(&status).unwrap();
        fs::write(status_dir.join("agent-001.json"), json).unwrap();

        println!("Agent heartbeat written.");
        thread::sleep(interval);
    }
}

fn get_hostname() -> String {
    hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn get_kernel() -> String {
    Command::new("uname")
        .arg("-r")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn get_uptime_secs() -> u64 {
    fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split('.').next().map(|v| v.parse().unwrap_or(0)))
        .unwrap_or(0)
}
