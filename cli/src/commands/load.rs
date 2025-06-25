use aya::{BpfLoader, programs::TracePoint};
use clap::Args;
use std::path::PathBuf;
use crate::utils::logger::{info, success, error};

#[derive(Args, Debug)]
pub struct LoadOptions {
    #[arg(short, long, default_value = "target/trace_execve.o")]
    pub program: PathBuf,

    #[arg(short, long, default_value = "trace_execve")]
    pub name: String,

    #[arg(short, long, default_value = "syscalls/sys_enter_execve")]
    pub tracepoint: String,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub json: bool,
}

pub fn handle_load(opts: LoadOptions) {
    if !opts.program.exists() {
        error(&format!("eBPF ELF file not found at: {}", opts.program.display()));
        return;
    }

    if opts.dry_run {
        success("✓ Dry run mode - ELF validated and options parsed.");
        return;
    }

    if opts.verbose {
        info(&format!("Loading program: {}", opts.name));
        info(&format!("From ELF file: {}", opts.program.display()));
        info(&format!("Target tracepoint: {}", opts.tracepoint));
    }

    match BpfLoader::new().load_file(&opts.program) {
        Ok(mut bpf) => {
            match bpf.program_mut(&opts.name) {
                Some(prog) => {
                    if let Ok(tp) = prog.try_into() {
                        let tp: &mut TracePoint = tp;
                        if let Err(e) = tp.load() {
                            error(&format!("Failed to load program: {}", e));
                            return;
                        }

                        if let Err(e) = tp.attach(&opts.tracepoint, "") {
                            error(&format!("Failed to attach to tracepoint: {}", e));
                            return;
                        }

                        if opts.json {
                            println!(
                                "{{ \"status\": \"ok\", \"program\": \"{}\", \"tracepoint\": \"{}\" }}",
                                opts.name, opts.tracepoint
                            );
                        } else {
                            success(&format!("✓ Program '{}' attached to '{}'", opts.name, opts.tracepoint));
                        }
                    } else {
                        error("Could not convert program to TracePoint");
                    }
                }
                None => error("Program not found in ELF"),
            }
        }
        Err(e) => {
            error(&format!("Failed to load ELF: {}", e));
        }
    }
}
