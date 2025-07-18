use clap::Args;
use anyhow::{Result, Context};
use std::{fs, path::Path, process::Command};

#[derive(Args)]
pub struct UpdateAgentOptions {
    #[arg(long)]
    pub agent: String,

    #[arg(long)]
    pub version: Option<String>,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub restart: bool,
}

pub async fn handle_update_agent(opts: UpdateAgentOptions) -> Result<()> {
    let agent_bin_path = format!("/opt/eclipta/agents/{}/eclipta-agent", opts.agent);
    let new_bin_path = match &opts.version {
        Some(v) => format!("/opt/eclipta/releases/{}/eclipta-agent", v),
        None => "/opt/eclipta/releases/latest/eclipta-agent".into(),
    };

    println!("Updating agent '{}'...", opts.agent);
    println!(" - Current path: {}", agent_bin_path);
    println!(" - New version:  {}", new_bin_path);

    if !Path::new(&new_bin_path).exists() {
        anyhow::bail!("New agent binary not found at: {}", new_bin_path);
    }

    // Replace binary
    fs::copy(&new_bin_path, &agent_bin_path)
        .with_context(|| format!("Failed to copy new agent binary to {}", agent_bin_path))?;

    println!(" Agent '{}' binary updated.", opts.agent);

    // Optional restart
    if opts.restart {
        println!(" Restarting agent '{}'", opts.agent);
        Command::new("systemctl")
            .args(&["restart", &format!("eclipta-agent@{}", opts.agent)])
            .status()
            .context("Failed to restart agent service")?;
    }

    Ok(())
}
