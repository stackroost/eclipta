use crate::utils::logger::{info, error};
use aya::{Ebpf, maps::perf::PerfEventArray, util::online_cpus};
use bytes::BytesMut;
use clap::Args;
use std::{convert::TryInto, path::PathBuf, time::Duration};
use tokio::{signal, task, time};

#[derive(Args, Debug)]
pub struct LogOptions {
    #[arg(short, long, default_value = "target/trace_execve.o")]
    pub program: PathBuf,

    #[arg(short, long, default_value = "trace_execve_events")]
    pub map: String,
}

pub async fn handle_logs(opts: LogOptions) {
    if !opts.program.exists() {
        error("Missing compiled eBPF program. Run `eclipta load` first.");
        return;
    }

    let mut bpf = match Ebpf::load_file(&opts.program) {
        Ok(bpf) => bpf,
        Err(e) => {
            error(&format!("Failed to load ELF: {}", e));
            return;
        }
    };

    let map = match bpf.take_map(&opts.map) {
        Some(m) => m,
        None => {
            error(&format!("Map '{}' not found in program", opts.map));
            return;
        }
    };

    let mut perf_array: PerfEventArray<_> = match map.try_into() {
        Ok(pa) => pa,
        Err(_) => {
            error("Map is not a valid PerfEventArray");
            return;
        }
    };

    info("Listening for perf event logs...\nPress Ctrl+C to exit.\n");

    for cpu_id in online_cpus().unwrap() {
        let mut buf = match perf_array.open(cpu_id, None) {
            Ok(b) => b,
            Err(e) => {
                error(&format!("Failed to open perf buffer on CPU {}: {}", cpu_id, e));
                continue;
            }
        };

        task::spawn(async move {
            let mut buffers = vec![BytesMut::with_capacity(1024)];
            loop {
                match buf.read_events(&mut buffers) {
                    Ok(events) => {
                        for buf in &buffers[..events.read] {
                            let event = String::from_utf8_lossy(&buf);
                            println!("🟢 {}", event);
                        }
                        if events.lost > 0 {
                            error(&format!("Lost {} events due to buffer overflow", events.lost));
                        }
                    }
                    Err(e) => {
                        error(&format!("Failed to read events: {}", e));
                    }
                }
                time::sleep(Duration::from_millis(100)).await;
            }
        });
    }

    // Wait for Ctrl+C
    if let Err(e) = signal::ctrl_c().await {
        error(&format!("Failed to wait for Ctrl+C: {}", e));
    }

    println!("\n🛑 Exiting logs...");
}
