use std::{ fs, thread, time::Duration };
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

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
    cpu_load: [f32; 3],
    mem_used_mb: u64,
    mem_total_mb: u64,
    process_count: usize,
    disk_used_mb: u64,
    disk_total_mb: u64,
    net_rx_kb: u64,
    net_tx_kb: u64,
    tcp_connections: usize,
    alert: bool,
}

fn main() {
    let agent_id = "agent-001";
    let interval = Duration::from_secs(5);
    let status_dir = PathBuf::from("/run/eclipta");
    fs::create_dir_all(&status_dir).ok();

    loop {
        let hostname = get_hostname();
        let kernel = get_kernel();
        let version = "v0.1.0".to_string();
        let now = Utc::now().to_rfc3339();
        let uptime_secs = get_uptime_secs();

        let cpu_load = get_cpu_load();
        let (mem_used_mb, mem_total_mb) = get_memory_mb();
        let (disk_used_mb, disk_total_mb) = get_disk_usage();
        let (net_rx_kb, net_tx_kb) = get_network_io();
        let process_count = get_process_count();
        let tcp_connections = get_tcp_connection_count();

        let alert = should_alert(&cpu_load, mem_used_mb, mem_total_mb);
        fs::write(status_dir.join(format!("{agent_id}.pid")), std::process::id().to_string()).ok();

        let status = AgentStatus {
            id: agent_id.to_string(),
            hostname,
            kernel,
            version,
            last_seen: now,
            uptime_secs,
            cpu_load,
            mem_used_mb,
            mem_total_mb,
            process_count,
            disk_used_mb,
            disk_total_mb,
            net_rx_kb,
            net_tx_kb,
            tcp_connections,
            alert,
        };

        if let Ok(json) = serde_json::to_string_pretty(&status) {
            let path = status_dir.join(format!("{agent_id}.json"));
            if let Ok(mut file) = File::create(path) {
                let _ = file.write_all(json.as_bytes());
                println!("✅ Agent heartbeat written.");
            }
        }

        thread::sleep(interval);
    }
}

fn get_hostname() -> String {
    hostname::get().unwrap_or_default().to_string_lossy().to_string()
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
        .and_then(|s|
            s
                .split('.')
                .next()
                .map(|v| v.parse().unwrap_or(0))
        )
        .unwrap_or(0)
}

fn get_cpu_load() -> [f32; 3] {
    fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| {
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() >= 3 {
                Some([
                    parts[0].parse().unwrap_or(0.0),
                    parts[1].parse().unwrap_or(0.0),
                    parts[2].parse().unwrap_or(0.0),
                ])
            } else {
                None
            }
        })
        .unwrap_or([0.0, 0.0, 0.0])
}

fn get_memory_mb() -> (u64, u64) {
    let data = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total: u64 = 0;
    let mut free: u64 = 0;

    for line in data.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            free = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        }
    }

    let used = total.saturating_sub(free);
    (used / 1024, total / 1024) // return in MB
}

fn get_disk_usage() -> (u64, u64) {
    let output = Command::new("df").arg("-BM").arg("/").output();
    if let Ok(out) = output {
        let lines = String::from_utf8_lossy(&out.stdout);
        for line in lines.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let total = parts[1].trim_end_matches('M').parse().unwrap_or(0);
                let used = parts[2].trim_end_matches('M').parse().unwrap_or(0);
                return (used, total);
            }
        }
    }
    (0, 0)
}

fn get_network_io() -> (u64, u64) {
    let data = fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let mut rx = 0;
    let mut tx = 0;
    for line in data.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 17 {
            rx += parts[1].parse::<u64>().unwrap_or(0); // RX bytes
            tx += parts[9].parse::<u64>().unwrap_or(0); // TX bytes
        }
    }
    (rx / 1024, tx / 1024) // Return in KB
}

fn get_process_count() -> usize {
    let entries = match fs::read_dir("/proc") {
        Ok(entries) => entries,
        Err(_) => {
            return 0;
        }
    };

    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .chars()
                .all(|c| c.is_ascii_digit())
        })
        .count()
}

fn get_tcp_connection_count() -> usize {
    fs::read_to_string("/proc/net/tcp")
        .map(|content| content.lines().skip(1).count())
        .unwrap_or(0)
}

fn should_alert(cpu: &[f32; 3], used: u64, total: u64) -> bool {
    let mem_percent = ((used as f32) / (total as f32)) * 100.0;
    cpu[0] > 2.0 || mem_percent > 90.0
}
