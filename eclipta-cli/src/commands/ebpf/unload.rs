use crate::utils::logger::{success, error, info};
use crate::utils::paths::{default_bin_object, default_state_path};
use crate::utils::state::{load_state, save_state};
use aya::Ebpf;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct UnloadOptions {
    /// Path to eBPF ELF (defaults to $ECLIPTA_BIN or ./bin/ebpf.so)
    #[arg(short, long)]
    pub program: Option<PathBuf>,

    /// Program name inside ELF
    #[arg(short, long)]
    pub name: Option<String>,

    /// Tracepoint in the form "category:name" or "category/name"
    #[arg(short = 't', long)]
    pub tracepoint: Option<String>,

    /// State file to update (default XDG local data dir)
    #[arg(long)]
    pub state_file: Option<PathBuf>,

    /// Unpin pinned objects from bpffs
    #[arg(long)]
    pub unpin: bool,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub fn handle_unload(opts: UnloadOptions) {
    let program_path = opts.program.unwrap_or_else(default_bin_object);
    let state_file = opts.state_file.unwrap_or_else(default_state_path);

    if !program_path.exists() {
        error("Missing compiled eBPF program.");
        return;
    }

    let name = if let Some(n) = opts.name.clone() { n } else {
        // fallback to last record from state
        let st = load_state(&state_file);
        if let Some(last) = st.attachments.last() {
            last.name.clone()
        } else {
            error("No program name provided and no state available.");
            return;
        }
    };

    if opts.verbose { info(&format!("Attempting to unload program: {}", name)); }

    let bpf = match Ebpf::load_file(&program_path) {
        Ok(bpf) => bpf,
        Err(e) => {
            error(&format!("Failed to load ELF: {}", e));
            return;
        }
    };

    if bpf.program(&name).is_none() {
        error("Program not found in ELF");
        return;
    }

    // Update state: remove records matching name
    let mut st = load_state(&state_file);
    let removed: Vec<_> = st.attachments.iter().filter(|r| r.name == name).cloned().collect();
    st.attachments.retain(|r| r.name != name);
    let _ = save_state(&state_file, st);

    if opts.unpin {
        for rec in removed {
            if let Some(pp) = rec.pinned_prog { let _ = std::fs::remove_file(pp); }
            for m in rec.pinned_maps { let _ = std::fs::remove_file(m); }
        }
    }

    if opts.json {
        println!("{{ \"status\": \"ok\", \"unloaded\": true, \"program\": \"{}\" }}", name);
    } else {
        success(&format!("✓ Unloaded program '{}'", name));
    }
}