use aya::{Ebpf, programs::TracePoint};
use std::convert::TryInto;
use std::error::Error;
use std::fs;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    // Load the eBPF ELF at runtime
    let data = fs::read("../target/trace_execve.o")?;
   let mut bpf = Ebpf::load(&data)?;


    let program: &mut TracePoint = bpf
        .program_mut("trace_execve")
        .ok_or("program not found")?
        .try_into()?;

    program.load()?;
    program.attach("syscalls", "sys_enter_execve")?;

    println!("trace_execve attached to sys_enter_execve");

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
