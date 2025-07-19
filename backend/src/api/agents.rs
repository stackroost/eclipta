use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

use std::path::PathBuf;

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
    #[serde(default)]
    pub active: bool,
}

pub fn routes() -> Router {
    Router::new()
        .route("/", get(get_all_agents_handler))
        .route("/kill/:id", post(kill_agent_handler))
}

async fn get_all_agents_handler() -> Json<Vec<Agent>> {
    let agents = load_all_agents_from_run().await;
    Json(agents)
}

async fn kill_agent_handler(Path(id): Path<String>) -> impl IntoResponse {
    let output = Command::new("eclipta")
        .arg("kill-agent")
        .arg("--agent")
        .arg(&id)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                (StatusCode::OK, format!("Agent {} killed successfully", id))
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "Failed to kill agent {}: {}",
                        id,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                )
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to execute kill command for agent {}: {}", id, e),
        ),
    }
}

async fn load_all_agents_from_run() -> Vec<Agent> {
    let mut agents = vec![];

    for entry in glob("/run/eclipta/*.json").expect("Failed to read glob pattern") {
        if let Ok(path) = entry {
            if path.is_file() {
                if let Ok(contents) = fs::read_to_string(&path) {
                    match serde_json::from_str::<Agent>(&contents) {
                        Ok(mut agent) => {
                            let pid_path = PathBuf::from(format!("/run/eclipta/{}.pid", agent.id));
                            agent.active = pid_path.exists();
                            agents.push(agent);
                        }
                        Err(e) => eprintln!("Failed to parse {}: {}", path.display(), e),
                    }
                }
            }
        }
    }

    agents
}
