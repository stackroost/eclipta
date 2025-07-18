use clap::Args;
use std::{fs, path::PathBuf};
use crate::utils::logger::{error, success};

#[derive(Args, Debug)]
pub struct RestartAgentOptions {
    /// Agent ID like agent-001
    pub id: String,
}

pub fn handle_restart_agent(opts: RestartAgentOptions) {
    let trigger_file = PathBuf::from(format!("/run/eclipta/restart-{}.trigger", opts.id));

    match fs::write(&trigger_file, b"restart") {
        Ok(_) => {
            success(&format!("Restart signal sent to agent `{}`", opts.id));
        }
        Err(e) => {
            error(&format!("Failed to create trigger file: {}", e));
        }
    }
}
