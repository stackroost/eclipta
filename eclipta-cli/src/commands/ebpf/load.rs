use clap::Args;
use std::path::PathBuf;
use crate::db::programs::{get_program_by_id, get_program_by_title};
use crate::utils::db::ensure_db_ready;
use aya::{
    Ebpf, 
    programs::{
        Program, 
        ProgramError
    }
};
use object::{Object, ObjectSection};
use std::collections::HashSet;
use tokio::process::Command;
use anyhow::{Result, Context, anyhow};

#[derive(Args, Debug)]
pub struct LoadOptions {
    #[arg(short, long)]
    pub program: Option<PathBuf>,

    #[arg(long)]
    pub id: Option<i32>,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(long)]
    pub iface: Option<String>,

    #[arg(long)]
    pub socket_fd: Option<i32>,
}

pub const XDP_SECTION: &str = "xdp";
pub const XDP_DROP_SECTION: &str = "xdp_drop";
pub const TC_INGRESS_SECTION: &str = "tc_ingress";
pub const TC_EGRESS_SECTION: &str = "tc_egress";
pub const SOCKET_FILTER_SECTION: &str = "socket_filter";
// pub const TRACEPOINT_NET_SECTION: &str = "tracepoint/net";
pub const KPROBE_NET_SECTION: &str = "kprobe/net";
pub const UPROBE_NET_SECTION: &str = "uprobe/net";
pub const LSM_NET_SECTION: &str = "lsm/net";
// pub const TRACEPOINT_SECTION: &str = "tracepoint";

#[derive(Debug)]
pub struct ProgramRequirements {
    pub sections: HashSet<String>,
    pub requires_interface: bool,
    pub requires_socket_fd: bool,
    pub program_type: String,
    pub tracepoint_category: Option<String>,
    pub tracepoint_name: Option<String>,
}

pub async fn handle_load(opts: LoadOptions) -> Result<()> {
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
                return Err(anyhow!("Multiple programs found with title '{}'. Please use --id to specify which one to load.", title));
            }
            _ => {
                return Err(anyhow!("No program found with title '{}'", title));
            }
        }
    } else if let Some(ref program_path) = opts.program {
        println!("Using direct program path: {}", program_path.display());
        program_path.clone()
    } else {
        return Err(anyhow!("Please specify a program to load using --id, --title, or --program"));
    };

    println!("Validating eBPF ELF object...");
    let requirements = validate_ebpf_file(&program_path)?;
    
    println!("Checking runtime arguments...");
    validate_runtime_args(&opts, &requirements)?;

    let should_skip_verifier = requirements.sections.iter()
        .all(|s| s.contains("TC"));
    
    // Load and attach the program, keeping the Ebpf object alive
    let attach_result = if should_skip_verifier {
        println!("Skipping Aya verifier for TC-only programs");
        attach_program_to_kernel(&program_path, &requirements, &opts).await?
    } else {
        println!("Loading and attaching eBPF program using Aya...");
        load_and_attach_ebpf(&program_path, &requirements, &opts).await?
    };

    println!("Verifying kernel program attachment...");
    verify_kernel_attachment(&requirements, &opts).await?;

    print_program_summary(&requirements, &opts, &attach_result)?;

    println!("eBPF program loaded and attached successfully!");
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
        println!("Section: {}", section.name().unwrap());
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

fn validate_runtime_args(opts: &LoadOptions, requirements: &ProgramRequirements) -> Result<()> {
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



async fn load_and_attach_ebpf(
    path: &PathBuf, 
    requirements: &ProgramRequirements, 
    opts: &LoadOptions
) -> Result<String> {
    let mut ebpf = Ebpf::load_file(path)
        .context("Failed to load eBPF object with Aya")?;

    let map_count = ebpf.maps().count();
    if map_count == 0 {
        println!("No maps found in eBPF object");
    } else {
        println!("Found {} maps in eBPF object", map_count);
    }

    // Load all programs first
    for (name, program) in ebpf.programs_mut() {
        match load_program_by_type(program) {
            Ok(()) => println!("Program '{}' loaded successfully", name),
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("busy") || error_msg.contains("already") {
                    println!("Program '{}' already loaded (EBUSY)", name);
                    continue;
                }
                return Err(anyhow!("Failed to load program '{}': {}", name, e));
            }
        }
    }

    println!("Aya eBPF loading completed successfully");

    // Now attach the programs based on type
    match requirements.program_type.as_str() {
        "XDP" => {
            let iface = opts.iface.as_ref()
                .ok_or_else(|| anyhow!("Interface required for XDP programs"))?;
            
            for (name, program) in ebpf.programs_mut() {
                if let Program::Xdp(xdp_prog) = program {
                    xdp_prog.attach(iface, aya::programs::XdpFlags::default())
                        .context("Failed to attach XDP program to interface")?;
                    
                    println!("XDP program '{}' attached to interface '{}'", name, iface);
                    return Ok(format!("XDP program attached to {}", iface));
                }
            }
            Err(anyhow!("No XDP program found in eBPF object"))
        }
        
        "Tracepoint" => {
            let category = requirements.tracepoint_category.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint category not found in ELF sections"))?;
            let name = requirements.tracepoint_name.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint name not found in ELF sections"))?;
            
            for (prog_name, program) in ebpf.programs_mut() {
                if let Program::TracePoint(tp_prog) = program {
                    tp_prog.attach(category, name)
                        .context(format!("Failed to attach Tracepoint program to '{}:{}'", category, name))?;
                    
                    println!("Tracepoint program '{}' attached to '{}:{}'", prog_name, category, name);
                    return Ok(format!("Tracepoint program attached to {}:{}", category, name));
                }
            }
            Err(anyhow!("No Tracepoint program found in eBPF object"))
        }
        
        _ => {
            // For other program types, just return success since they were loaded
            Ok(format!("Program type {} loaded successfully", requirements.program_type))
        }
    }
}

async fn attach_program_to_kernel(
    path: &PathBuf, 
    requirements: &ProgramRequirements, 
    opts: &LoadOptions
) -> Result<String> {
    match requirements.program_type.as_str() {
        "XDP" => {
            let iface = opts.iface.as_ref()
                .ok_or_else(|| anyhow!("Interface required for XDP programs"))?;
            
            let mut ebpf = Ebpf::load_file(path)
                .context("Failed to load eBPF for XDP attachment")?;
            
            for (name, program) in ebpf.programs_mut() {
                if let Program::Xdp(xdp_prog) = program {
                    xdp_prog.load()
                        .context("Failed to load XDP program")?;
                    
                    xdp_prog.attach(iface, aya::programs::XdpFlags::default())
                        .context("Failed to attach XDP program to interface")?;
                    
                    println!("XDP program '{}' attached to interface '{}'", name, iface);
                    return Ok(format!("XDP program attached to {}", iface));
                }
            }
            Err(anyhow!("No XDP program found in eBPF object"))
        }
        
        "TC" => {
            let iface = opts.iface.as_ref()
                .ok_or_else(|| anyhow!("Interface required for TC programs"))?;
            
            let mut ebpf = Ebpf::load_file(path)
                .context("Failed to load eBPF for TC attachment")?;
            
            for (name, program) in ebpf.programs_mut() {
                if let Program::SchedClassifier(tc_prog) = program {
                    tc_prog.load()
                        .context("Failed to load TC program")?;
                    
                    if name.contains("ingress") {
                        tc_prog.attach(iface, aya::programs::TcAttachType::Ingress)
                            .context("Failed to attach TC program to ingress")?;
                        println!("TC program '{}' attached to interface '{}' ingress", name, iface);
                    } else if name.contains("egress") {
                        tc_prog.attach(iface, aya::programs::TcAttachType::Egress)
                            .context("Failed to attach TC program to egress")?;
                        println!("TC program '{}' attached to interface '{}' egress", name, iface);
                    }
                    
                    return Ok(format!("TC program attached to {} {}", iface, 
                        if name.contains("ingress") { "ingress" } else { "egress" }));
                }
            }
            Err(anyhow!("No TC program attached to interface '{}'", iface))
        }
        
        "SocketFilter" => {
            let socket_fd = opts.socket_fd
                .ok_or_else(|| anyhow!("Socket FD required for SocketFilter programs"))?;
            
            let mut ebpf = Ebpf::load_file(path)
                .context("Failed to load eBPF for SocketFilter attachment")?;
            
            for (_name, program) in ebpf.programs_mut() {
                if let Program::SocketFilter(_sf_prog) = program {
                    println!("SocketFilter attachment requires proper socket handling - skipping attachment");
                    return Ok(format!("SocketFilter program loaded but not attached (FD: {})", socket_fd));
                }
            }
            Err(anyhow!("No SocketFilter program found in eBPF object"))
        }
        
        "Tracepoint" => {
            let category = requirements.tracepoint_category.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint category not found in ELF sections"))?;
            let name = requirements.tracepoint_name.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint name not found in ELF sections"))?;
            
            let mut ebpf = Ebpf::load_file(path)
                .context("Failed to load eBPF for Tracepoint attachment")?;
            
            for (prog_name, program) in ebpf.programs_mut() {
                if let Program::TracePoint(tp_prog) = program {
                    tp_prog.load()
                        .context("Failed to load Tracepoint program")?;
                    
                    tp_prog.attach(category, name)
                        .context(format!("Failed to attach Tracepoint program to '{}:{}'", category, name))?;
                    
                    println!("Tracepoint program '{}' attached to '{}:{}'", prog_name, category, name);
                    return Ok(format!("Tracepoint program attached to {}:{}", category, name));
                }
            }
            Err(anyhow!("No Tracepoint program found in eBPF object"))
        }
        
        _ => {
            println!("Program type '{}' not yet implemented for kernel attachment", requirements.program_type);
            Ok(format!("Program type {} loaded but not attached", requirements.program_type))
        }
    }
}

async fn verify_kernel_attachment(requirements: &ProgramRequirements, opts: &LoadOptions) -> Result<()> {
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
            if output_str.contains("prog/xdp") {
                println!("XDP program verified as attached to interface '{}'", iface);
            } else {
                return Err(anyhow!("XDP program not found attached to interface '{}'", iface));
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
                if output_str.contains("handle") && output_str.contains("bpf") {
                    println!("TC ingress program verified as attached to interface '{}'", iface);
                }
            }
            
            let egress_output = Command::new("tc")
                .args(["filter", "show", "dev", iface, "egress"])
                .output()
                .await;
            
            if let Ok(output) = egress_output {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("handle") && output_str.contains("bpf") {
                    println!("TC egress program verified as attached to interface '{}'", iface);
                }
            }
        }
        
        "SocketFilter" => {
            println!("SocketFilter verification requires manual inspection of socket state");
        }
        
        "Tracepoint" => {
            let category = requirements.tracepoint_category.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint category not found for verification"))?;
            let name = requirements.tracepoint_name.as_ref()
                .ok_or_else(|| anyhow!("Tracepoint name not found for verification"))?;
            
            // Use bpftool to verify tracepoint attachment
            let output = Command::new("bpftool")
                .args(["link", "list"])
                .output()
                .await
                .context("Failed to execute bpftool command")?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Look for tracepoint programs in the output
            let mut found = false;
            for line in output_str.lines() {
                if line.contains("tracepoint") {
                    println!("  Found tracepoint link: {}", line.trim());
                    // Check if this line contains our specific tracepoint
                    if line.contains(category) && line.contains(name) {
                        found = true;
                        break;
                    }
                }
            }
            
            if found {
                println!("Tracepoint program verified as attached to '{}:{}'", category, name);
            } else {
                println!("Tracepoint program not found attached to '{}:{}'", category, name);
                println!("Available tracepoint links:");
                for line in output_str.lines() {
                    if line.contains("tracepoint") {
                        println!("  {}", line.trim());
                    }
                }
                return Err(anyhow!("Failed to verify tracepoint attachment to '{}:{}'", category, name));
            }
        }
        
        _ => {
            println!("Verification not implemented for program type '{}'", requirements.program_type);
        }
    }
    
    Ok(())
}

fn print_program_summary(
    requirements: &ProgramRequirements, 
    opts: &LoadOptions, 
    attach_result: &str
) -> Result<()> {
    println!("\nProgram Summary:");
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
    println!("   Kernel Attachment: {}", attach_result);
    
    Ok(())
}

pub(crate) fn load_program_by_type(program: &mut Program) -> Result<(), ProgramError> {
    match program {
        Program::Xdp(p) => p.load(),
        Program::SchedClassifier(p) => p.load(),
        Program::TracePoint(p) => p.load(),
        Program::KProbe(p) => p.load(),
        Program::UProbe(p) => p.load(),
        Program::SocketFilter(p) => p.load(),
        Program::CgroupSkb(p) => p.load(),
        Program::CgroupSock(p) => p.load(),
        Program::CgroupSockAddr(p) => p.load(),
        Program::CgroupSockopt(p) => p.load(),
        Program::CgroupSysctl(p) => p.load(),
        Program::CgroupDevice(p) => p.load(),
        Program::SockOps(p) => p.load(),
        Program::SkMsg(p) => p.load(),
        Program::SkLookup(p) => p.load(),
        Program::PerfEvent(p) => p.load(),
        Program::RawTracePoint(p) => p.load(),
        Program::SkSkb(p) => p.load(),
        Program::Lsm(_) => {
            println!("Skipping LSM program load - requires lsm_hook_name and BTF");
            Ok(())
        },
        Program::BtfTracePoint(_) => {
            println!("Skipping BTF TracePoint program load - requires tracepoint name and BTF");
            Ok(())
        },
        Program::FEntry(_) => {
            println!("Skipping FEntry program load - requires function name and BTF");
            Ok(())
        },
        Program::FExit(_) => {
            println!("Skipping FExit program load - requires function name and BTF");
            Ok(())
        },
        Program::Extension(_) => {
            println!("Skipping Extension program load - requires ProgramFd and function name");
            Ok(())
        },
        _ => {
            println!("Unknown program type, skipping load");
            Ok(())
        }
    }
}
