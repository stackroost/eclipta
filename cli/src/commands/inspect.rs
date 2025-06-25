use aya::{Ebpf};
use clap::Args;
use std::path::PathBuf;
use crate::utils::logger::{info, error};
use serde_json;


#[derive(Args, Debug)]
pub struct InspectOptions {
    #[arg(short, long, default_value = "target/trace_execve.o")]
    pub program: PathBuf,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub fn handle_inspect(opts: InspectOptions) {
    if !opts.program.exists() {
        error("Missing compiled eBPF program.");
        return;
    }

    let bpf = match Ebpf::load_file(&opts.program) {
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
            "elf": opts.program.display().to_string(),
            "programs": programs,
            "maps": maps,
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        info(&format!("✅ ELF: {}", opts.program.display()));
        println!("Programs:");
        for name in &programs {
            println!("  - {}", name);
        }

        println!("Maps:");
        for name in &maps {
            println!("  - {}", name);
        }
    }
}
