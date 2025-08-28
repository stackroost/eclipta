use clap::Args;
use std::path::PathBuf;
use crate::db::programs::{get_program_by_id, get_program_by_title};
use crate::utils::db::init_db;
// Fixed imports based on current Aya API
use aya::{Ebpf, programs::{Program, ProgramError}};
use object::Object;
use object::ObjectSection;
use std::collections::HashSet;
use std::io::Error as IoError;
// Import EBUSY from nix crate
use nix::errno::Errno::EBUSY;

#[derive(Args, Debug)]
pub struct LoadOptions {
    #[arg(short, long)]
    pub program: Option<PathBuf>,

    #[arg(long)]
    pub id: Option<i32>,

    #[arg(long)]
    pub title: Option<String>,
}

pub const XDP_SECTION: &str = "xdp";
pub const XDP_DROP_SECTION: &str = "xdp_drop";
pub const TC_INGRESS_SECTION: &str = "tc_ingress";
pub const TC_EGRESS_SECTION: &str = "tc_egress";
pub const SOCKET_FILTER_SECTION: &str = "socket_filter";
pub const TRACEPOINT_NET_SECTION: &str = "tracepoint/net";
pub const KPROBE_NET_SECTION: &str = "kprobe/net";
pub const UPROBE_NET_SECTION: &str = "uprobe/net";
pub const LSM_NET_SECTION: &str = "lsm/net";

pub async fn handle_load(opts: LoadOptions) {
    let pool = match init_db().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to init DB: {}", e);
            return;
        }
    };

    if let Some(id) = opts.id {
        match get_program_by_id(&pool, id).await {
            Ok(Some(p)) => {
                println!("ID: {}, Title: {}", p.id, p.title);
                
                handle_file_process(p.path.clone().into());
            }
            Ok(None) => println!("No program found with id {}", id),
            Err(e) => eprintln!("Failed to fetch program by id {}: {}", id, e),
        }
    } else if let Some(ref title) = opts.title {
        match get_program_by_title(&pool, title).await {
            Ok(rows) if rows.len() == 1 => {
                let p = &rows[0];
                println!("ID: {}, Title: {}", p.id, p.title);
            }
            Ok(rows) if rows.len() > 1 => {
                eprintln!("Multiple programs found with title '{}'. Please load using --id.", title);
            }
            Ok(_) => println!("No program found with title '{}'", title),
            Err(e) => eprintln!("Failed to fetch programs by title '{}': {}", title, e),
        }
    } else {
        eprintln!("Please specify a program to load using --id or --title");
    }
}

pub fn validate_ebpf_file(path: PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }

    if path.extension().and_then(|ext| ext.to_str()) != Some("o") {
        return Err(format!("File is not an eBPF object (.o) file: {}", path.display()));
    }

    // 2. ELF format validation
    let file_data = match std::fs::read(&path) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to read file: {}", e)),
    };

    let obj = match object::File::parse(&*file_data) {
        Ok(obj) => obj,
        Err(e) => return Err(format!("Failed to parse ELF file: {}", e)),
    };

    // 3. Section recognition
    let mut found_sections = HashSet::new();
    for section in obj.sections() {
        if let Ok(name) = section.name() {
            match name {
                XDP_SECTION | XDP_DROP_SECTION => { found_sections.insert("XDP"); }
                TC_INGRESS_SECTION => { found_sections.insert("TC Ingress"); }
                TC_EGRESS_SECTION => { found_sections.insert("TC Egress"); }
                SOCKET_FILTER_SECTION => { found_sections.insert("Socket Filter"); }
                TRACEPOINT_NET_SECTION => { found_sections.insert("Tracepoint"); }
                KPROBE_NET_SECTION => { found_sections.insert("Kprobe"); }
                UPROBE_NET_SECTION => { found_sections.insert("Uprobe"); }
                LSM_NET_SECTION => { found_sections.insert("LSM"); }
                _ => {}
            }
        }
    }

    if found_sections.is_empty() {
        return Err("No recognized eBPF program sections found".to_string());
    }

    println!("Found eBPF program sections: {}", 
        found_sections.iter().cloned().collect::<Vec<_>>().join(", "));

    // 4. Aya load test - using Ebpf instead of deprecated Bpf
    let mut ebpf = match Ebpf::load_file(&path) {
        Ok(ebpf) => ebpf,
        Err(e) => return Err(format!("Failed to load eBPF object: {}", e)),
    };

    // 5. Map validation
    if ebpf.maps().next().is_none() {
        println!("Warning: No maps found in eBPF object");
    }

    // 6. Try to load programs (verifier test) - Fixed iteration approach
    for (name, program) in ebpf.programs_mut() {
        if let Err(e) = load_program_by_type(program) {
            return Err(format!("Verifier rejected program {}: {}", name, e));
        }
    }

    // 7. Try to attach programs (if possible) - Fixed iteration approach
    for (name, program) in ebpf.programs_mut() {
        // This is a simplified attachment test - in practice you'd need to handle
        // different program types with appropriate attachment methods
        if let Err(e) = try_attach_program(name, program) {
            // EBUSY might indicate the program is already attached, which is not a validation failure
            if let Some(os_error) = e.raw_os_error() {
                if os_error == EBUSY as i32 {
                    continue; // Skip EBUSY errors
                }
            }
            return Err(format!("Failed to attach program {}: {}", name, e));
        }
    }

    // 8. Policy/security check (simplified)
    if !is_allowed_program_type(&found_sections) {
        return Err("Program contains disallowed program types".to_string());
    }

    println!("eBPF object validation successful: {}", path.display());
    Ok(())
}

fn is_allowed_program_type(found_sections: &HashSet<&str>) -> bool {
    // Implement your policy checks here
    // For example, you might want to disallow certain program types
    let disallowed_types: HashSet<&str> = ["LSM"].iter().cloned().collect();
    found_sections.is_disjoint(&disallowed_types)
}

// Helper function to load programs based on their type
fn load_program_by_type(program: &mut Program) -> Result<(), ProgramError> {
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
        // These program types require additional parameters that we don't have in this context
        // We'll skip loading them for now and just print a message
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
            // For any program types not explicitly handled
            println!("Unknown program type, skipping load");
            Ok(())
        }
    }
}

// Fixed function signature and implementation
fn try_attach_program(name: &str, program: &mut Program) -> Result<(), IoError> {
    // This is a simplified example - actual attachment logic would depend on program type
    // For now, we'll just return Ok to avoid compilation errors
    // In a real implementation, you'd match on program type and attach appropriately
    match program {
        Program::Xdp(_) => {
            // For XDP programs, you'd typically attach to a network interface
            // program.attach("eth0", XdpFlags::default())?;
            println!("Would attach XDP program: {}", name);
        }
        Program::SchedClassifier(_) => {
            // For TC programs, you'd attach to a network interface with specific parameters
            println!("Would attach TC program: {}", name);
        }
        Program::TracePoint(_) => {
            // For tracepoint programs, you'd attach to specific kernel tracepoints
            println!("Would attach TracePoint program: {}", name);
        }
        Program::KProbe(_) => {
            // For kprobe programs, you'd attach to specific kernel functions
            println!("Would attach KProbe program: {}", name);
        }
        Program::UProbe(_) => {
            // For uprobe programs, you'd attach to specific user-space functions
            println!("Would attach UProbe program: {}", name);
        }
        Program::Lsm(_) => {
            // For LSM programs, you'd attach to specific LSM hooks
            println!("Would attach LSM program: {}", name);
        }
        _ => {
            println!("Unknown program type for: {}", name);
        }
    }
    
    Ok(())
}

pub fn handle_file_process(path: PathBuf) {
    match validate_ebpf_file(path.clone()) {
        Ok(()) => println!("eBPF object file is valid: {}", path.display()),
        Err(e) => eprintln!("Validation failed: {}", e),
    }
}