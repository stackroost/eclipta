#![no_std]
#![no_main]

use aya_ebpf::{macros::xdp, programs::XdpContext};
use aya_ebpf::bindings::xdp_action;

#[xdp(name = "xdp_drop")]
pub fn xdp_drop(_ctx: XdpContext) -> u32 {
    xdp_action::XDP_DROP
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
