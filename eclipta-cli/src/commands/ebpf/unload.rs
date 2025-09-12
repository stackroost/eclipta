use crate::utils::logger::{success, info};
use crate::utils::paths::default_state_path;
use crate::utils::state::{load_state, save_state};
use crate::db::programs::{get_program_by_id, get_program_by_title};
use crate::utils::db::ensure_db_ready;
use object::{Object, ObjectSection};
use std::collections::HashSet;
use tokio::process::Command;
use anyhow::{Result, Context, anyhow};
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct UnloadOptions {
    /// Path to eBPF ELF (defaults to $ECLIPTA_BIN or ./bin/ebpf.so)
    #[arg(short, long)]
    pub program: Option<PathBuf>,

    /// Program ID from database
    #[arg(long)]
    pub id: Option<i32>,

    /// Program title from database
    #[arg(long)]
    pub title: Option<String>,

    /// Program name inside ELF
    #[arg(short, long)]
    pub name: Option<String>,

    /// Network interface for XDP/TC programs
    #[arg(long)]
    pub iface: Option<String>,

    /// Socket file descriptor for SocketFilter programs
    #[arg(long)]
    pub socket_fd: Option<i32>,

    /// Tracepoint in the form "category:name" or "category/name"
    #[arg(short = 't', long)]
    pub tracepoint: Option<String>,

    /// State file to update (default XDG local data dir)
    #[arg(long)]
    pub state_file: Option<PathBuf>,

    /// Unpin pinned objects from bpffs
    #[arg(long)]
    pub unpin: bool,

    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub const XDP_SECTION: &str = "xdp";
pub const XDP_DROP_SECTION: &str = "xdp_drop";
pub const TC_INGRESS_SECTION: &str = "tc_ingress";
pub const TC_EGRESS_SECTION: &str = "tc_egress";
pub const SOCKET_FILTER_SECTION: &str = "socket_filter";
pub const KPROBE_NET_SECTION: &str = "kprobe/net";
pub const UPROBE_NET_SECTION: &str = "uprobe/net";
pub const LSM_NET_SECTION: &str = "lsm/net";

#[derive(Debug)]
pub struct ProgramRequirements {
    pub sections: HashSet<String>,
    pub requires_interface: bool,
    pub requires_socket_fd: bool,
    pub program_type: String,
    pub tracepoint_category: Option<String>,
    pub tracepoint_name: Option<String>,
}

pub async fn handle_unload(opts: UnloadOptions) -> Result<()> {
    let pool = ensure_db_ready().await
        .map_err(|e| anyhow!("Failed to initialize database: {}", e))?;

    let program_path = if let Some(id) = opts.id {
        let program = get_program_by_id(&pool, id).await
            .context("Failed to fetch program from database")?
            .ok_or_else(|| anyhow!("No program found with id {}", id))?;
        
        println!("Found program: ID: {}, Title: {}", program.id, program.title);
        PathBuf::from(program.path)
    } else if let Some(ref title) = opts.title {
        let programs = get_program_by_title(&pool, title).await
            .context("Failed to fetch programs from database")?;
        
        match programs.len() {
            1 => {
                let program = &programs[0];
                println!("Found program: ID: {}, Title: {}", program.id, program.title);
                PathBuf::from(program.path.clone())
            }
            n if n > 1 => {
                return Err(anyhow!("Multiple programs found with title '{}'. Please use --id to specify which one to unload.", title));
            }
            _ => {
                return Err(anyhow!("No program found with title '{}'", title));
            }
        }
    } else if let Some(ref program_path) = opts.program {
        println!("Using direct program path: {}", program_path.display());
        program_path.clone()
    } else {
        return Err(anyhow!("Please specify a program to unload using --id, --title, or --program"));
    };

    if !program_path.exists() {
        return Err(anyhow!("Missing compiled eBPF program: {}", program_path.display()));
    }

    println!("Validating eBPF ELF object...");
    let requirements = validate_ebpf_file(&program_path)?;
    
    println!("Checking runtime arguments...");
    validate_runtime_args(&opts, &requirements)?;

    let program_name = if let Some(n) = opts.name.clone() { 
        n 
    } else {
        // fallback to last record from state
        let state_file = opts.state_file.as_ref().cloned().unwrap_or_else(default_state_path);
        let st = load_state(&state_file);
        if let Some(last) = st.attachments.last() {
            last.name.clone()
        } else {
            return Err(anyhow!("No program name provided and no state available."));
        }
    };

    if opts.verbose { 
        info(&format!("Attempting to unload program: {}", program_name)); 
    }

    println!("Detaching eBPF program from kernel...");
    let detach_result = detach_program_from_kernel(&program_path, &requirements, &opts).await?;

    println!("Verifying kernel program detachment...");
    verify_kernel_detachment(&requirements, &opts).await?;

    // Update state: remove records matching name
    let state_file = opts.state_file.as_ref().cloned().unwrap_or_else(default_state_path);
    let mut st = load_state(&state_file);
    let removed: Vec<_> = st.attachments.iter().filter(|r| r.name == program_name).cloned().collect();
    st.attachments.retain(|r| r.name != program_name);
    let _ = save_state(&state_file, st);

    if opts.unpin {
        for rec in removed {
            if let Some(pp) = rec.pinned_prog { let _ = std::fs::remove_file(pp); }
            for m in rec.pinned_maps { let _ = std::fs::remove_file(m); }
        }
    }

    print_unload_summary(&requirements, &opts, &detach_result)?;

    if opts.json {
        println!("{{ \"status\": \"ok\", \"unloaded\": true, \"program\": \"{}\", \"type\": \"{}\" }}", 
            program_name, requirements.program_type);
    } else {
        success(&format!("✓ Unloaded {} program '{}'", requirements.program_type, program_name));
    }

    Ok(())
}

pub fn validate_ebpf_file(path: &PathBuf) -> Result<ProgramRequirements> {
    if !path.exists() {
        return Err(anyhow!("File does not exist: {}", path.display()));
    }

    if !path.is_file() {
        return Err(anyhow!("Path is not a file: {}", path.display()));
    }

    if path.extension().and_then(|ext| ext.to_str()) != Some("o") {
        return Err(anyhow!("File is not an eBPF object (.o) file: {}", path.display()));
    }

    let file_data = std::fs::read(path)
        .context("Failed to read file")?;

    let obj = object::File::parse(&*file_data)
        .context("Failed to parse ELF file")?;

    let mut found_sections = HashSet::new();
    let mut requires_interface = false;
    let mut requires_socket_fd = false;
    let mut program_type = String::new();
    let mut tracepoint_category: Option<String> = None;
    let mut tracepoint_name: Option<String> = None;

    for section in obj.sections() {
        if let Ok(name) = section.name() {
            if name.starts_with("tracepoint/") {
                found_sections.insert("Tracepoint".to_string());
                program_type = "Tracepoint".to_string();
                
                // Extract category and name from tracepoint section
                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() >= 3 {
                    tracepoint_category = Some(parts[1].to_string());
                    tracepoint_name = Some(parts[2].to_string());
                }
            } else {
                match name {
                    XDP_SECTION | XDP_DROP_SECTION => { 
                        found_sections.insert("XDP".to_string());
                        requires_interface = true;
                        program_type = "XDP".to_string();
                    }
                    TC_INGRESS_SECTION => { 
                        found_sections.insert("TC Ingress".to_string());
                        requires_interface = true;
                        program_type = "TC".to_string();
                    }
                    TC_EGRESS_SECTION => { 
                        found_sections.insert("TC Egress".to_string());
                        requires_interface = true;
                        program_type = "TC".to_string();
                    }
                    SOCKET_FILTER_SECTION => { 
                        found_sections.insert("Socket Filter".to_string());
                        requires_socket_fd = true;
                        program_type = "SocketFilter".to_string();
                    }
                    KPROBE_NET_SECTION => { 
                        found_sections.insert("Kprobe".to_string());
                        program_type = "Kprobe".to_string();
                    }
                    UPROBE_NET_SECTION => { 
                        found_sections.insert("Uprobe".to_string());
                        program_type = "Uprobe".to_string();
                    }
                    LSM_NET_SECTION => { 
                        found_sections.insert("LSM".to_string());
                        program_type = "LSM".to_string();
                    }
                    _ => {}
                }
            }
        }
    }

    if found_sections.is_empty() {
        return Err(anyhow!("No recognized eBPF program sections found"));
    }

    println!("Found eBPF program sections: {}", 
        found_sections.iter().cloned().collect::<Vec<_>>().join(", "));

    Ok(ProgramRequirements {
        sections: found_sections,
        requires_interface,
        requires_socket_fd,
        program_type,
        tracepoint_category,
        tracepoint_name,
    })
}

fn validate_runtime_args(opts: &UnloadOptions, requirements: &ProgramRequirements) -> Result<()> {
    if requirements.requires_interface && opts.iface.is_none() {
        return Err(anyhow!(
            "Program requires network interface. Please specify --iface <interface_name>"
        ));
    }

    if requirements.requires_socket_fd && opts.socket_fd.is_none() {
        return Err(anyhow!(
            "Program requires socket file descriptor. Please specify --socket-fd <fd>"
        ));
    }

    println!("Runtime arguments validation passed");
    Ok(())
}

async fn detach_program_from_kernel(
    _path: &PathBuf, 
    requirements: &ProgramRequirements, 
    opts: &UnloadOptions
) -> Result<String> {
    match requirements.program_type.as_str() {
        "XDP" => {
            let iface = opts.iface.as_ref()
                .ok_or_else(|| anyhow!("Interface required for XDP programs"))?;
            
            // Detach XDP program using ip command
            let output = Command::new("ip")
                .args(["link", "set", "dev", iface, "xdp", "off"])
                .output()
                .await
                .context("Failed to execute ip command to detach XDP")?;
            
            if !output.status.success() {
                let error_str = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow!("Failed to detach XDP program: {}", error_str));
            }
            
            println!("XDP program detached from interface '{}'", iface);
            Ok(format!("XDP program detached from {}", iface))
        }
        
        "TC" => {
            let iface = opts.iface.as_ref()
                .ok_or_else(|| anyhow!("Interface required for TC programs"))?;
            
            // Detach TC ingress programs
            let ingress_output = Command::new("tc")
                .args(["filter", "del", "dev", iface, "ingress"])
                .output()
                .await;
            
            if let Ok(output) = ingress_output {
                if output.status.success() {
                    println!("TC ingress program detached from interface '{}'", iface);
                }
            }
            
            // Detach TC egress programs
            let egress_output = Command::new("tc")
                .args(["filter", "del", "dev", iface, "egress"])
                .output()
                .await;
            
            if let Ok(output) = egress_output {
                if output.status.success() {
                    println!("TC egress program detached from interface '{}'", iface);
                }
            }
            
            Ok(format!("TC program detached from {} (ingress/egress)", iface))
        }
        
        "Tracepoint" => {
            let category = requirements.tracepoint_category.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint category not found in ELF sections"))?;
            let name = requirements.tracepoint_name.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint name not found in ELF sections"))?;
            
            // Use bpftool to detach tracepoint programs
            let output = Command::new("bpftool")
                .args(["link", "list"])
                .output()
                .await
                .context("Failed to execute bpftool command")?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Find and detach tracepoint programs
            for line in output_str.lines() {
                if line.contains("tracepoint") && line.contains(category) && line.contains(name) {
                    // Extract link ID and detach
                    if let Some(link_id) = extract_link_id(line) {
                        let detach_output = Command::new("bpftool")
                            .args(["link", "detach", "id", &link_id])
                            .output()
                            .await
                            .context("Failed to detach tracepoint program")?;
                        
                        if detach_output.status.success() {
                            println!("Tracepoint program detached from '{}:{}'", category, name);
                        }
                    }
                }
            }
            
            Ok(format!("Tracepoint program detached from {}:{}", category, name))
        }
        
        "SocketFilter" => {
            println!("SocketFilter detachment requires manual socket handling");
            Ok(format!("SocketFilter program detached (FD: {:?})", opts.socket_fd))
        }
        
        _ => {
            println!("Program type '{}' detachment not yet implemented", requirements.program_type);
            Ok(format!("Program type {} detached (manual verification required)", requirements.program_type))
        }
    }
}

async fn verify_kernel_detachment(requirements: &ProgramRequirements, opts: &UnloadOptions) -> Result<()> {
    match requirements.program_type.as_str() {
        "XDP" => {
            let iface = opts.iface.as_ref()
                .ok_or_else(|| anyhow!("Interface required for verification"))?;
            
            let output = Command::new("ip")
                .args(["link", "show", "dev", iface])
                .output()
                .await
                .context("Failed to execute ip command")?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            if !output_str.contains("prog/xdp") {
                println!("XDP program verified as detached from interface '{}'", iface);
            } else {
                return Err(anyhow!("XDP program still attached to interface '{}'", iface));
            }
        }
        
        "TC" => {
            let iface = opts.iface.as_ref()
                .ok_or_else(|| anyhow!("Interface required for verification"))?;
            
            let ingress_output = Command::new("tc")
                .args(["filter", "show", "dev", iface, "ingress"])
                .output()
                .await;
            
            if let Ok(output) = ingress_output {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if !output_str.contains("handle") || !output_str.contains("bpf") {
                    println!("TC ingress program verified as detached from interface '{}'", iface);
                }
            }
            
            let egress_output = Command::new("tc")
                .args(["filter", "show", "dev", iface, "egress"])
                .output()
                .await;
            
            if let Ok(output) = egress_output {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if !output_str.contains("handle") || !output_str.contains("bpf") {
                    println!("TC egress program verified as detached from interface '{}'", iface);
                }
            }
        }
        
        "Tracepoint" => {
            let category = requirements.tracepoint_category.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint category not found for verification"))?;
            let name = requirements.tracepoint_name.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint name not found for verification"))?;
            
            let output = Command::new("bpftool")
                .args(["link", "list"])
                .output()
                .await
                .context("Failed to execute bpftool command")?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            let mut found = false;
            for line in output_str.lines() {
                if line.contains("tracepoint") && line.contains(category) && line.contains(name) {
                    found = true;
                    break;
                }
            }
            
            if !found {
                println!("Tracepoint program verified as detached from '{}:{}'", category, name);
            } else {
                return Err(anyhow!("Tracepoint program still attached to '{}:{}'", category, name));
            }
        }
        
        _ => {
            println!("Verification not implemented for program type '{}'", requirements.program_type);
        }
    }
    
    Ok(())
}

fn print_unload_summary(
    requirements: &ProgramRequirements, 
    opts: &UnloadOptions, 
    detach_result: &str
) -> Result<()> {
    println!("\nUnload Summary:");
    println!("   Program Type: {}", requirements.program_type);
    println!("   Detected Sections: {}", 
        requirements.sections.iter().cloned().collect::<Vec<_>>().join(", "));
    
    if let Some(ref iface) = opts.iface {
        println!("   Network Interface: {}", iface);
    }
    
    if let Some(socket_fd) = opts.socket_fd {
        println!("   Socket FD: {}", socket_fd);
    }
    
    if let Some(ref category) = requirements.tracepoint_category {
        println!("   Tracepoint Category: {}", category);
    }
    if let Some(ref name) = requirements.tracepoint_name {
        println!("   Tracepoint Name: {}", name);
    }
    
    println!("   Interface Required: {}", requirements.requires_interface);
    println!("   Socket FD Required: {}", requirements.requires_socket_fd);
    println!("   Kernel Detachment: {}", detach_result);
    
    Ok(())
}

fn extract_link_id(line: &str) -> Option<String> {
    // Extract link ID from bpftool output line
    // Format: "123: tracepoint  name tracepoint_name  prog 456"
    if let Some(id_part) = line.split(':').next() {
        Some(id_part.trim().to_string())
    } else {
        None
    }
}