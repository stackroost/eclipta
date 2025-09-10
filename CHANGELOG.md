# Changelog

## [0.1.0] - 2025-01-27

### Added
- Complete project restructure for open source release
- Comprehensive documentation in `eclipta.yaml`
- Sample eBPF programs for testing
- Proper `.gitignore` configuration
- MIT License
- Updated installation script
- Clean project structure

### Changed
- Removed unnecessary directories (`bin/`, `ebpf-demo/`, `tests/`, `target/`)
- Updated `Cargo.toml` workspace configuration
- Streamlined README.md
- Reorganized project layout

### Removed
- Old binary files and demo programs
- Unnecessary test files
- Build artifacts and temporary files

### Project Structure
```
eclipta/
├── eclipta-cli/          # Main CLI application
├── examples/ebpf/        # Sample eBPF programs
├── bin/                  # Compiled eBPF programs for testing
├── eclipta.yaml         # Comprehensive documentation
├── README.md            # Project overview
├── LICENSE              # MIT License
├── install.sh           # Installation script
└── .gitignore           # Git ignore rules
```

### Sample eBPF Programs
- `simple_trace.c` - Basic tracepoint program
- `simple_xdp.c` - Basic XDP program
- Compiled versions available in `bin/` directory

### Documentation
- Complete command reference in `eclipta.yaml`
- Installation instructions
- Usage examples
- Development guidelines
- Troubleshooting guide
