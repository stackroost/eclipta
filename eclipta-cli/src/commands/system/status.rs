use crate::utils::db::ensure_db_ready;
use crate::db::programs::{get_program_by_id, list_programs};
use clap::Args;
use std::fs;
use std::process::Command;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tokio::process::Command as TokioCommand;
use chrono::{DateTime, Utc};
use std::os::unix::process::ExitStatusExt;

#[derive(Args, Debug)]
pub struct StatusOptions {
    /// Show status for a specific program ID
    #[arg(long)]
    pub id: Option<i32>,

    /// Show detailed information
    #[arg(long)]
    pub detailed: bool,

    /// Show only programs with specific status
    #[arg(long)]
    pub status: Option<String>,

    /// Show real-time updates (refresh every 2 seconds)
    #[arg(long)]
    pub watch: bool,

    /// Output format: table, json, or summary
    #[arg(long, default_value = "table")]
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramStatus {
    pub id: i32,
    pub title: String,
    pub version: String,
    pub db_status: String,
    pub kernel_status: KernelStatus,
    pub attachment_status: AttachmentStatus,
    pub performance_metrics: PerformanceMetrics,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KernelStatus {
    pub loaded: bool,
    pub program_id: Option<u32>,
    pub program_type: Option<String>,
    pub memory_usage: Option<u64>,
    pub verification_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachmentStatus {
    pub attached: bool,
    pub attachment_type: Option<String>,
    pub target: Option<String>,
    pub hook_point: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub events_processed: Option<u64>,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<u64>,
    pub error_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub kernel_version: String,
    pub bpf_support: BpfSupport,
    pub system_resources: SystemResources,
    pub loaded_programs_count: usize,
    pub active_programs_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BpfSupport {
    pub bpf_fs_mounted: bool,
    pub debug_fs_mounted: bool,
    pub cap_sys_admin: bool,
    pub bpf_verifier_available: bool,
    pub btf_support: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu_cores: usize,
    pub memory_total_gb: f64,
    pub memory_available_gb: f64,
    pub uptime_seconds: u64,
}

pub async fn run_status(opts: StatusOptions) -> Result<()> {
    if opts.watch {
        return run_status_watch(opts).await;
    }

    if let Some(program_id) = opts.id {
        show_program_status(program_id, opts.detailed, &opts.format).await?;
    } else {
        show_system_status(&opts).await?;
    }

    Ok(())
}

async fn show_program_status(program_id: i32, detailed: bool, format: &str) -> Result<()> {
    let pool = ensure_db_ready().await
        .map_err(|e| anyhow!("Database error: {}", e))?;
    let program = get_program_by_id(&pool, program_id).await?
        .ok_or_else(|| anyhow!("Program with ID {} not found", program_id))?;

    let program_status = build_program_status(&program).await?;

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&program_status)?);
        }
        "summary" => {
            print_program_summary(&program_status);
        }
        _ => {
            print_program_table(&program_status, detailed);
        }
    }

    Ok(())
}

async fn show_system_status(opts: &StatusOptions) -> Result<()> {
    let pool = ensure_db_ready().await
        .map_err(|e| anyhow!("Database error: {}", e))?;
    let programs = list_programs(&pool).await?;
    
    let mut program_statuses = Vec::new();
    for program in &programs {
        if let Some(status) = build_program_status(program).await.ok() {
            if let Some(ref status_filter) = opts.status {
                if status.db_status == *status_filter {
                    program_statuses.push(status);
                }
            } else {
                program_statuses.push(status);
            }
        }
    }

    let system_status = build_system_status(&program_statuses).await?;

    match opts.format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "system": system_status,
                "programs": program_statuses
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "summary" => {
            print_system_summary(&system_status, &program_statuses);
        }
        _ => {
            print_system_table(&system_status, &program_statuses);
        }
    }

    Ok(())
}

async fn build_program_status(program: &crate::db::programs::Program) -> Result<ProgramStatus> {
    let kernel_status = get_kernel_status(&program.title).await?;
    let attachment_status = get_attachment_status(&program.title).await?;
    let performance_metrics = get_performance_metrics(&program.title).await?;

    Ok(ProgramStatus {
        id: program.id,
        title: program.title.clone(),
        version: program.version.clone(),
        db_status: program.status.clone(),
        kernel_status,
        attachment_status,
        performance_metrics,
        last_updated: Utc::now(),
    })
}

async fn get_kernel_status(program_name: &str) -> Result<KernelStatus> {
    // Check if program is loaded in kernel using bpftool
    let output = TokioCommand::new("bpftool")
        .args(["prog", "list"])
        .output()
        .await
        .unwrap_or_else(|_| std::process::Output {
            stdout: vec![],
            stderr: vec![],
            status: std::process::ExitStatus::from_raw(1),
        });

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut loaded = false;
    let mut program_id = None;
    let mut program_type = None;
    let mut memory_usage = None;

    for line in stdout.lines() {
        if line.contains(program_name) {
            loaded = true;
            
            // Extract program ID
            if let Some(id_str) = line.split(':').next() {
                if let Ok(id) = id_str.parse::<u32>() {
                    program_id = Some(id);
                }
            }

            // Extract program type
            if let Some(type_part) = line.split_whitespace().nth(1) {
                program_type = Some(type_part.to_string());
            }

            // Extract memory usage
            if let Some(mem_part) = line.split("memlock").nth(1) {
                if let Some(mem_str) = mem_part.split('B').next() {
                    if let Ok(mem) = mem_str.parse::<u64>() {
                        memory_usage = Some(mem);
                    }
                }
            }
            break;
        }
    }

    Ok(KernelStatus {
        loaded,
        program_id,
        program_type,
        memory_usage,
        verification_status: if loaded { "verified".to_string() } else { "not_loaded".to_string() },
    })
}

async fn get_attachment_status(program_name: &str) -> Result<AttachmentStatus> {
    // Check if program is attached to any hooks
    let output = TokioCommand::new("bpftool")
        .args(["link", "list"])
        .output()
        .await
        .unwrap_or_else(|_| std::process::Output {
            stdout: vec![],
            stderr: vec![],
            status: std::process::ExitStatus::from_raw(1),
        });

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut attached = false;
    let mut attachment_type = None;
    let mut target = None;
    let mut hook_point = None;

    for line in stdout.lines() {
        if line.contains(program_name) {
            attached = true;
            
            // Extract attachment type
            if let Some(type_part) = line.split_whitespace().nth(1) {
                attachment_type = Some(type_part.to_string());
            }

            // Extract target/hook information
            if let Some(target_part) = line.split_whitespace().nth(2) {
                target = Some(target_part.to_string());
            }

            // Extract hook point
            if let Some(hook_part) = line.split_whitespace().nth(3) {
                hook_point = Some(hook_part.to_string());
            }
            break;
        }
    }

    Ok(AttachmentStatus {
        attached,
        attachment_type,
        target,
        hook_point,
    })
}

async fn get_performance_metrics(_program_name: &str) -> Result<PerformanceMetrics> {
    // This would integrate with actual eBPF program metrics
    // For now, return placeholder data
    Ok(PerformanceMetrics {
        events_processed: Some(0),
        cpu_usage: Some(0.0),
        memory_usage: Some(0),
        error_count: Some(0),
    })
}

async fn build_system_status(program_statuses: &[ProgramStatus]) -> Result<SystemStatus> {
    let kernel_version = get_kernel_version()?;
    let bpf_support = check_bpf_support()?;
    let system_resources = get_system_resources()?;
    
    let loaded_programs_count = program_statuses.iter()
        .filter(|p| p.kernel_status.loaded)
        .count();
    
    let active_programs_count = program_statuses.iter()
        .filter(|p| p.attachment_status.attached)
        .count();

    Ok(SystemStatus {
        kernel_version,
        bpf_support,
        system_resources,
        loaded_programs_count,
        active_programs_count,
    })
}

fn get_kernel_version() -> Result<String> {
    let output = Command::new("uname").arg("-r").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn check_bpf_support() -> Result<BpfSupport> {
    let bpf_fs_mounted = fs::metadata("/sys/fs/bpf").is_ok();
    let debug_fs_mounted = fs::metadata("/sys/kernel/debug").is_ok();
    let uid = nix::unistd::Uid::effective();
    let cap_sys_admin = uid.is_root();
    
    // Check BTF support
    let btf_support = fs::metadata("/sys/kernel/btf/vmlinux").is_ok();
    
    // Check BPF verifier availability
    let bpf_verifier_available = fs::metadata("/sys/kernel/debug/bpf/verifier_log").is_ok();

    Ok(BpfSupport {
        bpf_fs_mounted,
        debug_fs_mounted,
        cap_sys_admin,
        bpf_verifier_available,
        btf_support,
    })
}

fn get_system_resources() -> Result<SystemResources> {
    let cpu_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    
    let memory_total_gb = if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        if let Some(line) = meminfo.lines().find(|l| l.starts_with("MemTotal:")) {
            if let Some(kb_str) = line.split_whitespace().nth(1) {
                if let Ok(kb) = kb_str.parse::<u64>() {
                    kb as f64 / 1024.0 / 1024.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        }
    } else {
        0.0
    };

    let memory_available_gb = if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        if let Some(line) = meminfo.lines().find(|l| l.starts_with("MemAvailable:")) {
            if let Some(kb_str) = line.split_whitespace().nth(1) {
                if let Ok(kb) = kb_str.parse::<u64>() {
                    kb as f64 / 1024.0 / 1024.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        }
    } else {
        0.0
    };

    let uptime_seconds = if let Ok(uptime) = fs::read_to_string("/proc/uptime") {
        if let Some(seconds_str) = uptime.split_whitespace().next() {
            seconds_str.parse::<u64>().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    Ok(SystemResources {
        cpu_cores,
        memory_total_gb,
        memory_available_gb,
        uptime_seconds,
    })
}

fn print_program_summary(status: &ProgramStatus) {
    println!("\n\x1b[1;35mProgram Status Summary\x1b[0m");
    println!("ID: {}", status.id);
    println!("Title: {}", status.title);
    println!("Version: {}", status.version);
    println!("Database Status: {}", status.db_status);
    println!("Kernel Status: {}", if status.kernel_status.loaded { "LOADED" } else { "NOT LOADED" });
    println!("Attachment Status: {}", if status.attachment_status.attached { "ATTACHED" } else { "NOT ATTACHED" });
}

fn print_program_table(status: &ProgramStatus, detailed: bool) {
    println!("\n\x1b[1;35mProgram Status Details\x1b[0m");
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Program ID: {} │ Title: {}", status.id, status.title);
    println!("│ Version: {} │ Status: {}", status.version, status.db_status);
    println!("└─────────────────────────────────────────────────────────────┘");

    if detailed {
        println!("\n\x1b[1;36mKernel Status:\x1b[0m");
        println!("  Loaded: {}", if status.kernel_status.loaded { "✅ YES" } else { "❌ NO" });
        if let Some(id) = status.kernel_status.program_id {
            println!("  Kernel Program ID: {}", id);
        }
        if let Some(prog_type) = &status.kernel_status.program_type {
            println!("  Program Type: {}", prog_type);
        }
        if let Some(mem) = status.kernel_status.memory_usage {
            println!("  Memory Usage: {} bytes", mem);
        }

        println!("\n\x1b[1;36mAttachment Status:\x1b[0m");
        println!("  Attached: {}", if status.attachment_status.attached { "✅ YES" } else { "❌ NO" });
        if let Some(attach_type) = &status.attachment_status.attachment_type {
            println!("  Attachment Type: {}", attach_type);
        }
        if let Some(target) = &status.attachment_status.target {
            println!("  Target: {}", target);
        }
        if let Some(hook) = &status.attachment_status.hook_point {
            println!("  Hook Point: {}", hook);
        }

        println!("\n\x1b[1;36mPerformance Metrics:\x1b[0m");
        if let Some(events) = status.performance_metrics.events_processed {
            println!("  Events Processed: {}", events);
        }
        if let Some(cpu) = status.performance_metrics.cpu_usage {
            println!("  CPU Usage: {:.2}%", cpu);
        }
        if let Some(mem) = status.performance_metrics.memory_usage {
            println!("  Memory Usage: {} bytes", mem);
        }
        if let Some(errors) = status.performance_metrics.error_count {
            println!("  Error Count: {}", errors);
        }
    }
}

fn print_system_summary(system: &SystemStatus, programs: &[ProgramStatus]) {
    println!("\n\x1b[1;35mSystem Status Summary\x1b[0m");
    println!("Kernel: {}", system.kernel_version);
    println!("BPF Support: {}", if system.bpf_support.bpf_fs_mounted { "✅ Available" } else { "❌ Not Available" });
    println!("Loaded Programs: {}/{}", system.active_programs_count, system.loaded_programs_count);
    println!("Total Programs: {}", programs.len());
}

fn print_system_table(system: &SystemStatus, programs: &[ProgramStatus]) {
    println!("\n\x1b[1;35mECLIPTA CLI ▸ System Status\x1b[0m");
    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ Kernel Version: {}", system.kernel_version);
    println!("│ System Uptime: {} seconds", system.system_resources.uptime_seconds);
    println!("│ CPU Cores: {}", system.system_resources.cpu_cores);
    println!("│ Memory: {:.2} GB / {:.2} GB", 
        system.system_resources.memory_available_gb,
        system.system_resources.memory_total_gb);
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("\n\x1b[1;36mBPF System Support:\x1b[0m");
    println!("  BPF Filesystem: {}", if system.bpf_support.bpf_fs_mounted { "✅ Mounted" } else { "❌ Not Mounted" });
    println!("  Debug Filesystem: {}", if system.bpf_support.debug_fs_mounted { "✅ Mounted" } else { "❌ Not Mounted" });
    println!("  CAP_SYS_ADMIN: {}", if system.bpf_support.cap_sys_admin { "✅ Available" } else { "❌ Missing" });
    println!("  BTF Support: {}", if system.bpf_support.btf_support { "✅ Available" } else { "❌ Not Available" });
    println!("  BPF Verifier: {}", if system.bpf_support.bpf_verifier_available { "✅ Available" } else { "❌ Not Available" });

    println!("\n\x1b[1;36mProgram Status Overview:\x1b[0m");
    println!("  Total Programs: {}", programs.len());
    println!("  Loaded in Kernel: {}", system.loaded_programs_count);
    println!("  Active/Attached: {}", system.active_programs_count);

    if !programs.is_empty() {
        println!("\n\x1b[1;36mProgram Details:\x1b[0m");
        println!("┌─────┬─────────────────────┬──────────┬──────────┬─────────────┐");
        println!("│ ID  │ Title               │ Status   │ Kernel   │ Attached   │");
        println!("├─────┼─────────────────────┼──────────┼──────────┼─────────────┤");
        
        for program in programs {
            let kernel_status = if program.kernel_status.loaded { "✅" } else { "❌" };
            let attached_status = if program.attachment_status.attached { "✅" } else { "❌" };
            println!("│ {:3} │ {:19} │ {:8} │ {:8} │ {:10} │",
                program.id,
                if program.title.len() > 19 { &program.title[..19] } else { &program.title },
                program.db_status,
                kernel_status,
                attached_status
            );
        }
        println!("└─────┴─────────────────────┴──────────┴──────────┴─────────────┘");
    }
}

async fn run_status_watch(opts: StatusOptions) -> Result<()> {
    println!("\x1b[1;35mWatching system status... Press Ctrl+C to stop\x1b[0m");
    
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        
        // Show current status
        if let Some(program_id) = opts.id {
            show_program_status(program_id, opts.detailed, &opts.format).await?;
        } else {
            show_system_status(&opts).await?;
        }
        
        // Wait before next update
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
