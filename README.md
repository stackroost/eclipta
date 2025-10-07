# Eclipta CLI

A lightweight, modular CLI tool for managing and monitoring eBPF programs on Linux systems. Built in Rust for performance and reliability.

## Features

- **eBPF Program Management**: Load, unload, and monitor eBPF programs
- **System Monitoring**: Real-time system metrics and process monitoring
- **Interactive TUI**: Terminal-based user interface for monitoring
- **Configuration Management**: Flexible configuration system
- **Network Monitoring**: Network interface and traffic monitoring
- **Database Integration**: SQLite-based data storage and management

## Quick Start

### Prerequisites

- Rust 1.70+
- Linux kernel 4.18+ with eBPF support
- libbpf development libraries
- clang and llvm for eBPF compilation

### Installation

```bash
# Clone the repository
git clone https://github.com/stackroost/eclipta.git
cd eclipta

# Build from source
cargo build --release

# Install (requires sudo for eBPF operations)
sudo cp target/release/eclipta /usr/local/bin/eclipta
```

### Basic Usage

```bash
# Show welcome message and help
eclipta welcome

# Check system status
eclipta status

# Load a sample eBPF program
eclipta load --program bin/simple_trace.o --name my-tracer

# List loaded programs
eclipta list

# Start interactive monitoring
eclipta monitor

# Unload program
eclipta unload --program my-tracer
```

## Sample eBPF Programs

The project includes sample eBPF programs for testing:

- `bin/simple_trace.o` - Basic tracepoint program
- `bin/simple_xdp.o` - Basic XDP program

To build your own eBPF programs:

```bash
cd examples/ebpf
make
```

## Commands

### System Commands
- `welcome` - Show welcome message and setup help
- `status` - Show CLI runtime status
- `monitor` - Interactive terminal UI for monitoring
- `logs` - View system or agent logs
- `watch-cpu` - Monitor CPU usage

### eBPF Commands
- `load` - Load eBPF program
- `unload` - Unload eBPF program
- `list` - List loaded programs
- `inspect` - Inspect program details
- `upload` - Upload program to storage
- `remove` - Remove program from storage

### Network Commands
- `ping-all` - Check agent connectivity
- `alerts` - List alerting agents

### Configuration Commands
- `config` - Manage configuration
- `daemon` - Start daemon process

### Database Commands
- `check-db` - Check database status
- `migrate` - Run database migrations

## Configuration

Configuration is stored in `/etc/eclipta/config.yaml`. Key settings:

```yaml
log_level: "info"
daemon_enabled: false
auto_start_programs: false
monitoring_interval: 5
```

## Project Structure

```
eclipta/
├── eclipta-cli/          # Main CLI application
│   ├── src/
│   │   ├── commands/     # Command implementations
│   │   ├── utils/        # Utility functions
│   │   └── main.rs       # Entry point
│   └── Cargo.toml
├── examples/ebpf/        # Sample eBPF programs
│   ├── simple_trace.c
│   ├── simple_xdp.c
│   └── Makefile
├── bin/                  # Compiled eBPF programs
│   ├── simple_trace.o
│   └── simple_xdp.o
├── eclipta.yaml         # Comprehensive documentation
└── README.md
```

## Development

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Adding New Commands

1. Create command module in `eclipta-cli/src/commands/`
2. Add command to `main.rs` command enum
3. Implement command handler
4. Update documentation in `eclipta.yaml`

## Documentation

For comprehensive documentation including all commands, options, and examples, see `eclipta.yaml`.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT © 2025 Mahesh Bhatiya
