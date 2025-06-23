pub fn success(msg: &str) {
    println!("\x1b[1;32m✔\x1b[0m {}", msg);
}

pub fn info(msg: &str) {
    println!("\x1b[1;34mℹ\x1b[0m {}", msg);
}

pub fn warn(msg: &str) {
    println!("\x1b[1;33m⚠\x1b[0m {}", msg);
}

pub fn error(msg: &str) {
    eprintln!("\x1b[1;31m✖\x1b[0m {}", msg);
}
