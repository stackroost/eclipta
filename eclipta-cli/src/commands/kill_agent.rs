use clap::Args;
use std::{fs, path::PathBuf};
use anyhow::{Result, Context};

#[derive(Args)]
pub struct KillAgentOptions {
    #[arg(long)]
    pub agent: String,
}

pub fn handle_kill_agent(opts: KillAgentOptions) -> Result<()> {
    let pid_path = PathBuf::from(format!("/run/eclipta/{}.pid", opts.agent));
    
    let pid_str = fs::read_to_string(&pid_path)
        .with_context(|| format!("Failed to read PID file for agent '{}'", opts.agent))?;

    let pid: i32 = pid_str.trim().parse()
        .with_context(|| format!("Invalid PID found in {}", pid_path.display()))?;

    println!("Sending SIGTERM to agent '{}' (PID {})", opts.agent, pid);
    
    // Send SIGTERM to the process
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(pid),
        nix::sys::signal::Signal::SIGTERM,
    ).with_context(|| format!("Failed to send SIGTERM to PID {}", pid))?;

    println!("Kill signal sent successfully.");

    Ok(())
}
