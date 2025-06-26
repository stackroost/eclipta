use clap::Args;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::utils::logger::{error, info};
use tokio::time::{sleep, Duration};
use std::fs;
use std::ffi::OsStr;

#[derive(Args, Debug)]
pub struct AgentLogsOptions {
    /// Agent ID like agent-001 (optional)
    #[arg(long)]
    pub id: Option<String>,
}

pub async fn handle_agent_logs(opts: AgentLogsOptions) {
    let log_dir = PathBuf::from("/run/eclipta/logs");

    if !log_dir.exists() {
        error("Log directory not found: /run/eclipta/logs");
        return;
    }

    let files: Vec<PathBuf> = if let Some(id) = &opts.id {
        let p = log_dir.join(format!("{}.log", id));
        if !p.exists() {
            error(&format!("Log file not found: {}", p.display()));
            return;
        }
        vec![p]
    } else {
        match fs::read_dir(&log_dir) {
            Ok(entries) => entries
                .flatten()
                .filter(|e| e.path().extension() == Some(OsStr::new("log")))
                .map(|e| e.path())
                .collect(),
            Err(e) => {
                error(&format!("Failed to read log directory: {}", e));
                return;
            }
        }
    };

    for file in files {
        let file_name = file.file_name().unwrap().to_string_lossy().to_string();
        let f = File::open(&file).await.unwrap();
        let reader = BufReader::new(f);
        let mut lines = reader.lines();

        info(&format!("Tailing logs from: {}", file_name));

        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                println!(" [{}] {}", file_name, line);
                sleep(Duration::from_millis(200)).await;
            }
        });
    }

    // Keep the program running to tail
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
