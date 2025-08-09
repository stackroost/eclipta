#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::bpf_get_smp_processor_id,
    macros::{map, tracepoint},
    maps::HashMap,
    programs::TracePointContext,
};

#[map(name = "CPU_STATS")]
static mut CPU_STATS: HashMap<u32, u64> = HashMap::<u32, u64>::with_max_entries(256, 0);

#[tracepoint(name = "cpu_usage", category = "sched")]
pub fn cpu_usage(_ctx: TracePointContext) -> u32 {
    let cpu_id = unsafe { bpf_get_smp_processor_id() } as u32;

    unsafe {
        let current = match CPU_STATS.get(&cpu_id) {
            Some(v) => *v,
            None => 0,
        };
        let new_val = current.wrapping_add(1);
        let _ = CPU_STATS.insert(&cpu_id, &new_val, 0);
    }

    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
