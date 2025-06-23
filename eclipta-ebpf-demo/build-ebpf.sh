#!/bin/bash
set -e

# Build eBPF program from workspace root
target_dir="target/bpfel-unknown-none/release"

cargo build --release \
  --package ebpf \
  --target bpfel-unknown-none \
  -Z build-std=core

# Copy ELF for use by user-space
mkdir -p target
cp ${target_dir}/trace_execve target/trace_execve.o

echo "eBPF ELF ready at: target/trace_execve.o"