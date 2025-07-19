use axum::{routing::get, Json, Router};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Agent {
    pub id: String,
    pub hostname: String,
    pub version: String,
    pub cpu_load: [f64; 3],
    pub mem_used_mb: u64,
    pub disk_used_mb: u64,
    pub net_rx_kb: u64,
    pub alert: bool,
    pub last_seen: String,
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_all_agents_handler))
}

async fn get_all_agents_handler() -> Json<Vec<Agent>> {
    let agents = load_all_agents_from_run().await;
    Json(agents)
}

async fn load_all_agents_from_run() -> Vec<Agent> {
    let mut agents = vec![];

    for entry in glob("/run/eclipta/*.json").expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            if path.is_file() {
                if let Ok(contents) = fs::read_to_string(&path) {
                    match serde_json::from_str::<Agent>(&contents) {
                        Ok(agent) => agents.push(agent),
                        Err(e) => eprintln!("Failed to parse {}: {}", path.display(), e),
                    }
                }
            }
        }
    }

    agents
}
