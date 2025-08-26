#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action,
    macros::xdp,
    programs::XdpContext,
};

#[xdp]
pub fn simple_ebpf_program(ctx: XdpContext) -> u32 {
    match try_simple_ebpf_program(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_simple_ebpf_program(_ctx: XdpContext) -> Result<u32, u32> {
    // Simple success - no logging to avoid macro issues
    Ok(xdp_action::XDP_PASS) // Return 2 (XDP_PASS - allows packet through)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}