use aya::{EbpfLoader, maps::perf::PerfEventArray, util::online_cpus, programs::TracePoint};
use bytes::BytesMut;
use clap::Args;
use std::{convert::TryInto, path::PathBuf, time::Duration, mem};
use tokio::{signal, time};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExecEvent {
    pub pid: u32,
    pub comm: [u8; 16],
}

#[derive(Args, Debug)]
pub struct RunOptions {
    #[arg(short, long, default_value = "ebpf-demo/target/bpfel-unknown-none/release/libebpf.so")]
    pub program: PathBuf,

    #[arg(short, long, default_value = "trace_execve")]
    pub name: String,

    #[arg(short, long, default_value = "syscalls/sys_enter_execve")]
    pub tracepoint: String,

    #[arg(short = 'm', long, default_value = "trace_execve_events")]
    pub map: String,

    #[arg(long)]
    pub execve_format: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub async fn handle_run(opts: RunOptions) {
    if !opts.program.exists() {
        eprintln!("Missing compiled eBPF program at {}", opts.program.display());
        return;
    }

    let mut bpf = match EbpfLoader::new().load_file(&opts.program) {
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
            let parts: Vec<&str> = opts.tracepoint.split('/').collect();
            if parts.len() != 2 {
                eprintln!("Tracepoint must be 'category/name', got '{}'", opts.tracepoint);
                return;
            }
            if let Err(e) = tp.load() {
                eprintln!("Failed to load program: {}", e);
                return;
            }
            if let Err(e) = tp.attach(parts[0], parts[1]) {
                eprintln!("Failed to attach to tracepoint '{}': {}", opts.tracepoint, e);
                return;
            }
            if opts.verbose {
                println!("✓ Attached '{}' to '{}'", opts.name, opts.tracepoint);
            }
        }
        None => {
            eprintln!("Program '{}' not found in ELF", opts.name);
            return;
        }
    }

    let map = match bpf.take_map(&opts.map) {
        Some(m) => m,
        None => {
            eprintln!("Map '{}' not found in ELF", opts.map);
            return;
        }
    };
    let mut perf_array: PerfEventArray<_> = match map.try_into() {
        Ok(pa) => pa,
        Err(_) => {
            eprintln!("Map '{}' is not a PerfEventArray", opts.map);
            return;
        }
    };

    println!("Streaming events from '{}' (Ctrl+C to exit)", opts.map);

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

    if let Err(e) = signal::ctrl_c().await {
        eprintln!("Failed to wait for Ctrl+C: {}", e);
    }
} 