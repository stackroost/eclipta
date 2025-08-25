use aya::{programs::TracePoint, EbpfLoader};
use clap::Args;
use std::path::PathBuf;
use crate::utils::logger::{info, success, error};
use crate::utils::paths::{default_bin_object, default_pin_prefix, default_state_path};
use crate::utils::state::{AttachmentRecord, load_state, save_state};
use nix::sys::resource::{setrlimit, Resource, RLIM_INFINITY};
use nix::unistd::Uid;

#[derive(Args, Debug)]
pub struct LoadOptions {
    /// Path to eBPF ELF (defaults to $ECLIPTA_BIN or ./bin/ebpf.so)
    #[arg(short, long)]
    pub program: Option<PathBuf>,

    /// Program name inside ELF
    #[arg(short, long, default_value = "cpu_usage")]
    pub name: String,

    /// Tracepoint in the form "category:name" or "category/name" (e.g., "sched:sched_switch")
    #[arg(short = 't', long, default_value = "sched:sched_switch")]
    pub tracepoint: String,

    /// Pin the program and maps under a prefix in bpffs
    #[arg(long, default_value_t = true)]
    pub pin: bool,

    /// Pin prefix in bpffs (default $ECLIPTA_PIN_PATH or /sys/fs/bpf/eclipta)
    #[arg(long)]
    pub pin_prefix: Option<PathBuf>,

    /// Persist loader state to this file (default XDG local data dir)
    #[arg(long)]
    pub state_file: Option<PathBuf>,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(long)]
    pub json: bool,
}

pub fn handle_load(opts: LoadOptions) {
    let program_path = opts.program.unwrap_or_else(default_bin_object);
    if !program_path.exists() {
        error(&format!("eBPF ELF file not found at: {}", program_path.display()));
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
                "Program '{}' from '{}' would attach to '{}:{}' (pin: {})",
                opts.name,
                program_path.display(),
                tp_category,
                tp_name,
                opts.pin
            ));
        }
        return;
    }

    if opts.verbose {
        info(&format!("Loading program: {}", opts.name));
        info(&format!("From ELF file: {}", program_path.display()));
        info(&format!("Target tracepoint: {}:{}", tp_category, tp_name));
    }

    let pin_prefix = opts.pin_prefix.unwrap_or_else(default_pin_prefix);
    let state_file = opts.state_file.unwrap_or_else(default_state_path);

    match EbpfLoader::new().load_file(&program_path) {
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

                        let mut pinned_prog = None;
                        let mut pinned_maps = Vec::new();
                        if opts.pin {
                            let _ = std::fs::create_dir_all(&pin_prefix);
                            // Pin program
                            let prog_pin = pin_prefix.join(&opts.name);
                            if let Err(e) = tp.pin(&prog_pin) {
                                error(&format!("Failed to pin program: {}", e));
                            } else {
                                pinned_prog = Some(prog_pin);
                            }
                            // Pin maps
                            for (map_name, m) in bpf.maps_mut() {
                                let path = pin_prefix.join(map_name);
                                if let Err(e) = m.pin(&path) {
                                    if opts.verbose { info(&format!("Map '{}' pin failed: {}", map_name, e)); }
                                } else {
                                    pinned_maps.push(path);
                                }
                            }
                        }

                        // Save state
                        let mut st = load_state(&state_file);
                        st.attachments.push(AttachmentRecord {
                            name: opts.name.clone(),
                            kind: "tracepoint".to_string(),
                            trace_category: Some(tp_category.clone()),
                            trace_name: Some(tp_name.clone()),
                            pinned_prog,
                            pinned_maps,
                            pid: std::process::id(),
                            created_at: chrono::Utc::now().timestamp(),
                        });
                        let _ = save_state(&state_file, st);

                        if opts.json {
                            println!(
                                "{{ \"status\": \"ok\", \"program\": \"{}\", \"tracepoint\": \"{}:{}\", \"pinned\": {} }}",
                                opts.name, tp_category, tp_name, opts.pin
                            );
                        } else {
                            success(&format!("✓ Program '{}' attached to '{}:{}'", opts.name, tp_category, tp_name));
                            if opts.pin { info(&format!("Pinned under {}", pin_prefix.display())); }
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
