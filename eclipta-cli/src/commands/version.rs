use std::{fs, path::PathBuf};
use anyhow::Result;
use clap::Args;
use serde::Deserialize;

#[derive(Args)]
pub struct VersionOptions {
    #[arg(long)]
    pub agent: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AgentVersionInfo {
    id: String,
    version: String,
}

pub async fn handle_version(opts: VersionOptions) -> Result<()> {
    if let Some(agent_id) = opts.agent {
        let path = PathBuf::from(format!("/run/eclipta/{}.json", agent_id));

        if !path.exists() {
            println!("Could not find version info for agent '{}'", agent_id);
            return Ok(());
        }

        let contents = fs::read_to_string(&path)?;
        let agent_info: AgentVersionInfo = match serde_json::from_str(&contents) {
            Ok(info) => info,
            Err(_) => {
                println!("Malformed JSON for agent '{}'", agent_id);
                return Ok(());
            }
        };

        println!(" Agent '{}' version: {}", agent_info.id, agent_info.version);
    } else {
        println!(" eclipta CLI version: {}", env!("CARGO_PKG_VERSION"));
    }

    Ok(())
}
