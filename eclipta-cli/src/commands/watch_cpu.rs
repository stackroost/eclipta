use std::{fs, time::Duration};
use tokio::time::sleep;
use serde::Deserialize;
use chrono::Local;
use clap::Args;

#[derive(Deserialize)]
struct AgentCpuLoad {
    id: String,
    cpu_load: Option<[f32; 3]>,
    last_seen: String,
}

#[derive(Args)]
pub struct WatchCpuOptions {
    #[arg(long)]
    pub agent_id: Option<String>,
}

pub async fn handle_watch_cpu(opts: WatchCpuOptions) -> anyhow::Result<()> {
    let agent_filter = opts.agent_id;

    println!("Watching CPU load{} (Ctrl+C to quit)...",
        agent_filter
            .as_ref()
            .map(|id| format!(" for agent '{}'", id))
            .unwrap_or_default()
    );

    loop {
        let paths = fs::read_dir("/run/eclipta")?;
        println!("\n{}", Local::now().format("%H:%M:%S"));

        for entry in paths.flatten() {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(agent) = serde_json::from_str::<AgentCpuLoad>(&content) {
                    if let Some(ref filter) = agent_filter {
                        if agent.id != *filter {
                            continue;
                        }
                    }

                    if let Some(load) = agent.cpu_load {
                        println!(
                            "{} | CPU Load: {:.1} / {:.1} / {:.1} | Seen: {}",
                            agent.id,
                            load[0], load[1], load[2],
                            agent.last_seen
                        );
                    }
                }
            }
        }

        sleep(Duration::from_secs(3)).await;
    }
}
