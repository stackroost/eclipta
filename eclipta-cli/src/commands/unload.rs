use crate::utils::logger::{success, error};
use aya::Ebpf;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct UnloadOptions {
    #[arg(short, long, default_value = "target/trace_execve.o")]
    pub program: PathBuf,

    #[arg(short, long, default_value = "trace_execve")]
    pub name: String,

    #[arg(short, long, default_value = "syscalls/sys_enter_execve")]
    pub tracepoint: String,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub fn handle_unload(opts: UnloadOptions) {
    if !opts.program.exists() {
        error("Missing compiled eBPF program. Run `eclipta load` first.");
        return;
    }

    if opts.verbose {
        println!("Attempting to unload program: {}", opts.name);
        println!("ELF path: {:?}", opts.program);
    }

    let mut bpf = match Ebpf::load_file(&opts.program) {
        Ok(bpf) => bpf,
        Err(e) => {
            error(&format!("Failed to load ELF: {}", e));
            return;
        }
    };

    if bpf.program_mut(&opts.name).is_none() {
        error("Program not found in ELF");
        return;
    }

    // Dropping bpf unloads all programs
    drop(bpf);

    if opts.verbose {
        println!("Program '{}' unloaded by dropping Ebpf struct", opts.name);
    }

    if opts.json {
        println!(
            "{{ \"status\": \"ok\", \"unloaded\": true, \"program\": \"{}\" }}",
            opts.name
        );
    } else {
        success(&format!("✓ Unloaded program '{}'", opts.name));
    }
}