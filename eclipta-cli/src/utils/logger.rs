use colored::*;

pub fn info(msg: &str) {
    println!("{} {}", "[INFO]".blue().bold(), msg);
}

pub fn warn(msg: &str) {
    println!("{} {}", "[WARN]".yellow().bold(), msg);
}

pub fn success(msg: &str) {
    println!("{} {}", "[OK]".green().bold(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "[ERROR]".red().bold(), msg);
}
