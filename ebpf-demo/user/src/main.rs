use aya::{Bpf, programs::TracePoint};
use aya::maps::HashMap;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load the compiled eBPF object
    let mut bpf = Bpf::load_file(
        "target/bpfel-unknown-none/release/deps/ebpf.o"
    )?;

    // Attach to the sched:sched_switch tracepoint
    let program: &mut TracePoint = bpf.program_mut("cpu_usage")
        .unwrap()
        .try_into()?;
    program.load()?;
    program.attach("sched", "sched_switch")?;

    // Get the eBPF map for CPU stats
    let mut cpu_stats: HashMap<_, u32, u64> = HashMap::try_from(
        bpf.map_mut("CPU_STATS")?
    )?;

    println!("Tracking per-CPU usage in real time (Ctrl+C to exit)");
    loop {
        println!("---------------------------");
        for cpu_id in 0..num_cpus::get() as u32 {
            if let Ok(val) = cpu_stats.get(&cpu_id, 0) {
                println!("CPU {}: {} ns active", cpu_id, val);
            }
        }
        time::sleep(Duration::from_secs(1)).await;
    }
}
