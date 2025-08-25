use aya::{EbpfLoader, maps::perf::PerfEventArray, util::online_cpus, programs::TracePoint};
use bytes::BytesMut;
use clap::Args;
use std::{convert::TryInto, path::PathBuf, time::Duration, mem};
use tokio::{signal, time};
use crate::utils::paths::default_bin_object;
use nix::sys::resource::{setrlimit, Resource, RLIM_INFINITY};
use nix::unistd::Uid;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExecEvent {
    pub pid: u32,
    pub comm: [u8; 16],
}

#[derive(Args, Debug)]
pub struct RunOptions {
    /// Path to eBPF ELF (defaults to $ECLIPTA_BIN or ./bin/ebpf.so)
    #[arg(short, long)]
    pub program: Option<PathBuf>,

    /// Program name inside ELF
    #[arg(short, long, default_value = "cpu_usage")]
    pub name: String,

    /// Tracepoint in the form "category:name" or "category/name" (e.g., "sched:sched_switch")
    #[arg(short, long, default_value = "sched:sched_switch")]
    pub tracepoint: String,

    /// PerfEventArray map name to stream (optional)
    #[arg(short = 'm', long)]
    pub map: Option<String>,

    #[arg(long)]
    pub execve_format: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub async fn handle_run(opts: RunOptions) {
    if !Uid::effective().is_root() {
        eprintln!("This command must be run as root. Try: sudo eclipta run ...");
        return;
    }

    // Bump memlock to avoid failures on older kernels
    let _ = setrlimit(Resource::RLIMIT_MEMLOCK, RLIM_INFINITY, RLIM_INFINITY);

    let program_path = opts.program.clone().unwrap_or_else(default_bin_object);
    if !program_path.exists() {
        eprintln!("Missing compiled eBPF program at {}", program_path.display());
        return;
    }

    let mut bpf = match EbpfLoader::new().load_file(&program_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to load ELF: {}", e);
            return;
        }
    };

    match bpf.program_mut(&opts.name) {
        Some(p) => {
            let tp: &mut TracePoint = match p.try_into() {
                Ok(tp) => tp,
                Err(_) => {
                    eprintln!("Program '{}' is not a TracePoint", opts.name);
                    return;
                }
            };
            // Support both category:name and category/name
            let s = opts.tracepoint.replace('/', ":");
            let mut parts = s.splitn(2, ':');
            let cat = parts.next().unwrap_or("");
            let nam = parts.next().unwrap_or("");
            if cat.is_empty() || nam.is_empty() {
                eprintln!("Tracepoint must be 'category:name', got '{}'", opts.tracepoint);
                return;
            }
            if let Err(e) = tp.load() {
                eprintln!("Failed to load program: {}", e);
                return;
            }
            if let Err(e) = tp.attach(cat, nam) {
                eprintln!("Failed to attach to tracepoint '{}': {}", opts.tracepoint, e);
                return;
            }
            if opts.verbose {
                println!("✓ Attached '{}' to '{}:{}'", opts.name, cat, nam);
            }
        }
        None => {
            eprintln!("Program '{}' not found in ELF", opts.name);
            return;
        }
    }

    // If a map was provided, stream events; otherwise just wait until Ctrl+C
    if let Some(map_name) = opts.map.as_ref() {
        let map = match bpf.take_map(map_name) {
            Some(m) => m,
            None => {
                eprintln!("Map '{}' not found in ELF", map_name);
                return;
            }
        };
        let mut perf_array: PerfEventArray<_> = match map.try_into() {
            Ok(pa) => pa,
            Err(_) => {
                eprintln!("Map '{}' is not a PerfEventArray", map_name);
                return;
            }
        };

        println!("Streaming events from '{}' (Ctrl+C to exit)", map_name);

        for cpu_id in online_cpus().unwrap_or_default() {
            let mut buf = match perf_array.open(cpu_id, None) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Failed to open perf buffer on CPU {}: {}", cpu_id, e);
                    continue;
                }
            };

            let execve_fmt = opts.execve_format;
            tokio::spawn(async move {
                let mut bufs = vec![BytesMut::with_capacity(1024)];
                loop {
                    match buf.read_events(&mut bufs) {
                        Ok(events) => {
                            for rec in &bufs[..events.read] {
                                if execve_fmt && rec.len() >= mem::size_of::<ExecEvent>() {
                                    let ptr = rec.as_ptr() as *const ExecEvent;
                                    let ev = unsafe { *ptr };
                                    let comm = std::str::from_utf8(&ev.comm)
                                        .unwrap_or("")
                                        .trim_end_matches(char::from(0))
                                        .to_string();
                                    println!("exec pid={} comm={}", ev.pid, comm);
                                } else if let Ok(s) = std::str::from_utf8(&rec) {
                                    println!("{}", s.trim_end_matches(char::from(0)));
                                } else {
                                    println!(
                                        "{}",
                                        rec.iter()
                                            .map(|b| format!("{:02x}", b))
                                            .collect::<Vec<_>>()
                                            .join("")
                                    );
                                }
                            }
                            if events.lost > 0 {
                                eprintln!("Lost {} events (perf buffer overflow)", events.lost);
                            }
                        }
                        Err(e) => eprintln!("read_events error: {}", e),
                    }
                    time::sleep(Duration::from_millis(100)).await;
                }
            });
        }
    } else {
        println!("Attached. No map provided for streaming. Waiting (Ctrl+C to exit)...");
    }

    if let Err(e) = signal::ctrl_c().await {
        eprintln!("Failed to wait for Ctrl+C: {}", e);
    }
} 