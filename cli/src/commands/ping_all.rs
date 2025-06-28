use chrono::{DateTime, Utc};
use std::{fs, path::PathBuf};
use serde::Deserialize;
use humantime::format_duration;

#[derive(Deserialize)]
struct Agent {
    id: String,
    hostname: String,
    last_seen: String,
}

pub async fn handle_ping_all() {
    println!("Pinging all agents...\n");

    let dir = PathBuf::from("/run/eclipta");
    let mut online = 0;
    let mut offline = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(agent) = serde_json::from_str::<Agent>(&content) {
                    let seen = DateTime::parse_from_rfc3339(&agent.last_seen)
                        .unwrap_or_else(|_| Utc::now().into());
                    let seen = seen.with_timezone(&Utc);
                    let delta = Utc::now().signed_duration_since(seen);

                    let status = if delta.num_seconds() <= 10 {
                        online += 1;
                        format!("{} ({}) - ONLINE (last seen: {})",
                            agent.id, agent.hostname, format_duration(delta.to_std().unwrap()))
                    } else {
                        offline += 1;
                        format!("{} ({}) - OFFLINE (last seen: {})",
                            agent.id, agent.hostname, format_duration(delta.to_std().unwrap()))
                    };

                    println!("{}", status);
                }
            }
        }
    }

    println!("\nSummary: {} online, {} offline", online, offline);
}
