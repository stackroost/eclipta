use crate::utils::logger;
use std::fs;
use std::process::Command;

pub fn run_status() {
    println!("\n\x1b[1;35mECLIPTA CLI ▸ status\x1b[0m\n");

    let kernel = Command::new("uname").arg("-r").output().unwrap();
    let kernel_str = String::from_utf8_lossy(&kernel.stdout);
    logger::success(&format!("Kernel: {}", kernel_str.trim()));

    let has_bpf_fs = fs::metadata("/sys/fs/bpf").is_ok();
    logger::info(&format!("/sys/fs/bpf mounted: {}", if has_bpf_fs { "YES" } else { "NO" }));

    let has_debug_fs = fs::metadata("/sys/kernel/debug").is_ok();
    logger::info(&format!("/sys/kernel/debug mounted: {}", if has_debug_fs { "YES" } else { "NO" }));

    let uid = nix::unistd::Uid::effective();
    logger::info(&format!("CAP_SYS_ADMIN: {}", if uid.is_root() { "available" } else { "missing" }));

    let rustc = Command::new("rustc").arg("--version").output();
    if let Ok(r) = rustc {
        let v = String::from_utf8_lossy(&r.stdout);
        logger::info(&format!("Rust toolchain: {}", v.trim()));
    }

    logger::info("aya version: 0.13.1");
}
