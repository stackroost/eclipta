use std::time::Duration;
use tokio::time::sleep;
use clap::Args;
use sysinfo::{System, CpuRefreshKind, RefreshKind};

#[derive(Args)]
pub struct WatchCpuOptions {
    #[arg(long, default_value_t = 1)]
    pub interval_secs: u64,
}

pub async fn handle_watch_cpu(opts: WatchCpuOptions) -> anyhow::Result<()> {
    println!("Watching system CPU/memory (Ctrl+C to quit)...");

    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_memory(sysinfo::MemoryRefreshKind::everything()).with_cpu(CpuRefreshKind::everything()),
    );

    loop {
        sys.refresh_memory();
        sys.refresh_cpu();

        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let global_cpu = sys.global_cpu_info().cpu_usage();

        println!(
            "CPU: {:.1}% | Mem: {}/{} MiB",
            global_cpu,
            used_mem / 1024 / 1024,
            total_mem / 1024 / 1024
        );

        sleep(Duration::from_secs(opts.interval_secs)).await;
    }
}
