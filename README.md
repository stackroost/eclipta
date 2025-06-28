#  Eclipta – Self-Hosted Observability Platform for Linux Systems
![Rust](https://img.shields.io/badge/Rust-High%20Performance-orange)
![Status](https://img.shields.io/badge/status-Alpha-yellow)

> A fast, self-hosted, and modular observability toolkit written in Rust with support for CLI, agent daemon, live system metrics, and future eBPF integrations.

---

## Features


###  Agent Management
- `agents` – List and inspect all connected agents
- `inspect-agent` – View detailed info about a specific agent
- `restart-agent` – Restart an agent service
- `kill-agent` – Kill and stop an agent immediately
- `update-agent` – Upgrade agent binary (with version, force, restart flags)

###  System Monitoring
- `live` – Real-time terminal monitoring (CPU, memory, disk, etc.)
- `monitor` – Dashboard view of all agents
- `watch-cpu` – Live graph of CPU usage for one agent

### Metrics & Logs
- `logs` – View global system logs
- `agent-logs` – View logs for a specific agent (with tail/follow)
- `status` – Show system-wide status

### Configuration
- `config` – Get/set configuration for an agent (e.g. thresholds, intervals)
- `version` – Show current CLI version and agent version
- `alerts` – List agents in alert state

###  Utilities
- `load` / `unload` – Load or unload eBPF programs
- `inspect` – Inspect active kernel programs
- `daemon` – Run eclipta agent heartbeat writer in the background
- `ping-all` – Ping and verify all agent health
- `welcome` – Display the CLI welcome screen

---

## Built With

-  [Rust](https://www.rust-lang.org/)
-  [aya](https://github.com/aya-rs/aya) – eBPF for Rust
-  [tui](https://github.com/fdehau/tui-rs) – Terminal UI framework
-  [sysinfo](https://docs.rs/sysinfo) – System stats
-  [clap](https://docs.rs/clap) – Command-line interface
-  [chrono](https://docs.rs/chrono) – Timestamps and formatting

---
