#![no_std]
#![no_main]

use aya_ebpf::{macros::tracepoint, programs::TracePointContext};
use core::panic::PanicInfo;

#[tracepoint(name = "trace_execve", category = "syscalls")]
pub fn trace_execve(_ctx: TracePointContext) -> u32 {
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}