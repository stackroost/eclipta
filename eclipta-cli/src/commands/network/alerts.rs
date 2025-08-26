use std::{fs, path::PathBuf};
use anyhow::Result;
use serde::Deserialize;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Deserialize)]
struct AgentStatus {
    id: String,
    hostname: String,
    alert: Option<bool>,
    last_seen: Option<String>,
}

pub async fn handle_alerts() -> Result<()> {
    let dir = PathBuf::from("/run/eclipta");
    let mut alerted_agents = Vec::new();
    let now = Utc::now();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(agent) = serde_json::from_str::<AgentStatus>(&content) {
                    if let (Some(true), Some(last_seen_str)) = (agent.alert, &agent.last_seen) {
                        if let Ok(last_seen) = DateTime::parse_from_rfc3339(last_seen_str) {
                            let last_seen = last_seen.with_timezone(&Utc);
                            let age = now.signed_duration_since(last_seen);

                            if age < Duration::seconds(15) {
                                alerted_agents.push((agent.id, agent.hostname, last_seen_str.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    if alerted_agents.is_empty() {
        println!("No agents are currently in alert state.");
    } else {
        println!("Alerted Agents (last seen < 15s):\n");
        for (id, hostname, last_seen) in alerted_agents {
            println!("- [{}] {} | Last Seen: {}", id, hostname, last_seen);
        }
    }

    Ok(())
}
