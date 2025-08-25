use aya::Ebpf;
use clap::Args;
use std::path::PathBuf;
use crate::utils::logger::{info, error};
use serde_json;
use crate::utils::paths::default_bin_object;

#[derive(Args, Debug)]
pub struct InspectOptions {
    /// Path to eBPF ELF (defaults to $ECLIPTA_BIN or ./bin/ebpf.so)
    #[arg(short, long)]
    pub program: Option<PathBuf>,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub fn handle_inspect(opts: InspectOptions) {
    let program_path = opts.program.unwrap_or_else(default_bin_object);
    if !program_path.exists() {
        error("Missing compiled eBPF program.");
        return;
    }

    let bpf = match Ebpf::load_file(&program_path) {
        Ok(b) => b,
        Err(e) => {
            error(&format!("Failed to parse ELF: {}", e));
            return;
        }
    };

    let programs: Vec<_> = bpf.programs().map(|(name, _)| name.to_string()).collect();
    let maps: Vec<_> = bpf.maps().map(|(name, _)| name.to_string()).collect();

    if opts.json {
        let output = serde_json::json!({
            "elf": program_path.display().to_string(),
            "programs": programs,
            "maps": maps,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        info(&format!("✅ ELF: {}", program_path.display()));
        println!("Programs:");
        for name in &programs { println!("  - {}", name); }
        println!("Maps:");
        for name in &maps { println!("  - {}", name); }
    }
}
