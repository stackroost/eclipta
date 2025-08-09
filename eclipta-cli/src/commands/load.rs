use aya::{programs::TracePoint, EbpfLoader};
use clap::Args;
use std::path::PathBuf;
use crate::utils::logger::{info, success, error};
use nix::sys::resource::{setrlimit, Resource, RLIM_INFINITY};
use nix::unistd::Uid;

#[derive(Args, Debug)]
pub struct LoadOptions {
    #[arg(short, long, default_value = "/etc/eclipta/bin/ebpf.so")]
    pub program: PathBuf,

    #[arg(short, long, default_value = "cpu_usage")]
    pub name: String,

    /// Tracepoint in the form "category:name" or "category/name" (e.g., "sched:sched_switch")
    #[arg(short = 't', long, default_value = "sched:sched_switch")]
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

    if !Uid::effective().is_root() {
        error("This command must be run as root to create BPF maps and attach programs. Try: sudo eclipta load ...");
        return;
    }

    // Raise memlock limit to avoid map allocation failures on older kernels
    let _ = setrlimit(Resource::RLIMIT_MEMLOCK, RLIM_INFINITY, RLIM_INFINITY);

    // Parse tracepoint category/name
    let (tp_category, tp_name) = {
        let s = opts.tracepoint.replace('/', ":");
        let mut parts = s.splitn(2, ':');
        let cat = parts.next().unwrap_or("").trim().to_string();
        let nam = parts.next().unwrap_or("").trim().to_string();
        if cat.is_empty() || nam.is_empty() {
            error("Tracepoint must be in the form 'category:name' (e.g., 'sched:sched_switch')");
            return;
        }
        (cat, nam)
    };

    if opts.dry_run {
        success("✓ Dry run mode - ELF validated and options parsed.");
        if opts.verbose {
            info(&format!(
                "Program '{}' from '{}' would attach to '{}:{}'",
                opts.name,
                opts.program.display(),
                tp_category,
                tp_name
            ));
        }
        return;
    }

    if opts.verbose {
        info(&format!("Loading program: {}", opts.name));
        info(&format!("From ELF file: {}", opts.program.display()));
        info(&format!("Target tracepoint: {}:{}", tp_category, tp_name));
    }

    match EbpfLoader::new().load_file(&opts.program) {
        Ok(mut bpf) => {
            match bpf.program_mut(&opts.name) {
                Some(prog) => {
                    if let Ok(tp) = prog.try_into() {
                        let tp: &mut TracePoint = tp;
                        if let Err(e) = tp.load() {
                            error(&format!("Failed to load program: {}", e));
                            return;
                        }

                        if let Err(e) = tp.attach(&tp_category, &tp_name) {
                            error(&format!("Failed to attach to tracepoint: {}", e));
                            return;
                        }

                        if opts.json {
                            println!(
                                "{{ \"status\": \"ok\", \"program\": \"{}\", \"tracepoint\": \"{}:{}\" }}",
                                opts.name, tp_category, tp_name
                            );
                        } else {
                            success(&format!("✓ Program '{}' attached to '{}:{}'", opts.name, tp_category, tp_name));
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
