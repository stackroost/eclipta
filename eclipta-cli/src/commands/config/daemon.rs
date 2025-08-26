use chrono::Utc;
use serde::Serialize;
use std::{fs::File, io::Write, path::PathBuf, thread, time::Duration};
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind, LoadAvg};
use hostname::get;
use crate::utils::logger::success;

#[derive(Debug, Serialize)]
struct AgentStatus {
    id: String,
    hostname: String,
    kernel: String,
    version: String,
    uptime_secs: u64,
    cpu_load: [f32; 3],
    mem_used_mb: u64,
    mem_total_mb: u64,
    last_seen: String,
}

pub async fn handle_daemon() {
    let agent_id = "agent-001";
    let run_path = PathBuf::from(format!("/run/eclipta/{}.json", agent_id));

    success(" Starting Eclipta Agent Daemon...");

    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::everything()),
    );

    loop {
        sys.refresh_memory();
        sys.refresh_cpu();

        let hostname = get()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "unknown-host".into());

        let kernel = System::kernel_version().unwrap_or_else(|| "unknown-kernel".into());
        let version = "0.1.0".to_string();
        let uptime_secs = System::uptime();
        let load: LoadAvg = System::load_average();
        let mem_total_mb = sys.total_memory() / 1024;
        let mem_used_mb = (sys.total_memory() - sys.available_memory()) / 1024;
        let now = Utc::now().to_rfc3339();

        let agent = AgentStatus {
            id: agent_id.to_string(),
            hostname,
            kernel,
            version,
            uptime_secs,
            cpu_load: [
    load.one as f32,
    load.five as f32,
    load.fifteen as f32,
],
            mem_used_mb,
            mem_total_mb,
            last_seen: now,
        };

        if let Ok(json) = serde_json::to_string_pretty(&agent) {
            if let Ok(mut file) = File::create(&run_path) {
                let _ = file.write_all(json.as_bytes());
            }
        }

        thread::sleep(Duration::from_secs(5));
    }
}
