use std::path::{PathBuf};
use std::env;

pub fn default_bin_object() -> PathBuf {
    if let Ok(custom) = env::var("ECLIPTA_BIN") {
        let p = PathBuf::from(custom);
        if p.exists() { return p; }
    }
    if let Ok(home) = env::var("ECLIPTA_HOME") {
        let p = PathBuf::from(home).join("bin").join("ebpf.so");
        if p.exists() { return p; }
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let p = cwd.join("bin").join("ebpf.so");
    if p.exists() { return p; }

    // Fallback to executable dir/../bin/ebpf.so
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p = parent.join("..").join("bin").join("ebpf.so");
            if p.exists() { return p; }
        }
    }

    // Last resort: return cwd/bin/ebpf.so even if it doesn't exist
    cwd.join("bin").join("ebpf.so")
}

// pub fn default_pin_prefix() -> PathBuf {
//     if let Ok(p) = env::var("ECLIPTA_PIN_PATH") {
//         return PathBuf::from(p);
//     }
//     PathBuf::from("/sys/fs/bpf/eclipta")
// }

pub fn default_state_path() -> PathBuf {
    if let Ok(p) = env::var("ECLIPTA_STATE") { return PathBuf::from(p); }
    if let Some(dir) = dirs::data_local_dir() {
        return dir.join("eclipta").join("state.json");
    }
    PathBuf::from(".eclipta_state.json")
} 