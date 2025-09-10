#!/bin/bash

set -euo pipefail

echo "Installing Eclipta CLI..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed. Please install Rust first."
    echo "Visit: https://rustup.rs/"
    exit 1
fi

# Check if required dependencies are available
echo "Checking dependencies..."

# Check for libbpf
if ! pkg-config --exists libbpf; then
    echo "Warning: libbpf not found. You may need to install libbpf development libraries."
    echo "Ubuntu/Debian: sudo apt install libbpf-dev"
    echo "Arch Linux: sudo pacman -S libbpf"
    echo "Fedora: sudo dnf install libbpf-devel"
fi

# Check for clang
if ! command -v clang &> /dev/null; then
    echo "Warning: clang not found. You may need clang for eBPF compilation."
    echo "Ubuntu/Debian: sudo apt install clang llvm"
    echo "Arch Linux: sudo pacman -S clang llvm"
    echo "Fedora: sudo dnf install clang llvm"
fi

# Build the CLI
echo "Building Eclipta CLI..."
cargo build --release

# Create installation directory
INSTALL_DIR="/usr/local/bin"
echo "Installing to $INSTALL_DIR..."

# Copy binary (requires sudo)
if [ -w "$INSTALL_DIR" ]; then
    cp target/release/eclipta-cli "$INSTALL_DIR/eclipta"
    chmod +x "$INSTALL_DIR/eclipta"
else
    echo "Installing with sudo..."
    sudo cp target/release/eclipta-cli "$INSTALL_DIR/eclipta"
    sudo chmod +x "$INSTALL_DIR/eclipta"
fi

# Create configuration directory
CONFIG_DIR="/etc/eclipta"
if [ ! -d "$CONFIG_DIR" ]; then
    echo "Creating configuration directory..."
    if [ -w "/etc" ]; then
        mkdir -p "$CONFIG_DIR"
    else
        sudo mkdir -p "$CONFIG_DIR"
    fi
fi

# Create runtime directory
RUNTIME_DIR="/run/eclipta"
if [ ! -d "$RUNTIME_DIR" ]; then
    echo "Creating runtime directory..."
    if [ -w "/run" ]; then
        mkdir -p "$RUNTIME_DIR"
    else
        sudo mkdir -p "$RUNTIME_DIR"
    fi
fi

# Build sample eBPF programs
echo "Building sample eBPF programs..."
cd examples/ebpf
make install
cd ../..

echo ""
echo " Installation complete!"
echo ""
echo "Usage:"
echo "  eclipta welcome          # Show welcome message"
echo "  eclipta status           # Check system status"
echo "  eclipta load --help      # Load eBPF program"
echo "  eclipta monitor          # Start monitoring"
echo ""
echo "Sample eBPF programs are available in bin/:"
echo "  bin/simple_trace.o       # Basic tracepoint program"
echo "  bin/simple_xdp.o         # Basic XDP program"
echo ""
echo "For full documentation, see eclipta.yaml"
