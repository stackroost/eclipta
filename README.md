#  Eclipta – Self-Hosted Observability Platform for Linux Systems
![Rust](https://img.shields.io/badge/Rust-High%20Performance-orange)
![Status](https://img.shields.io/badge/status-Alpha-yellow)

> A fast, self-hosted, and modular observability toolkit written in Rust with support for CLI, agent daemon, live system metrics, and future eBPF integrations.

---

## Features

- **Modular CLI Interface**  
  Easily run commands like `load`, `logs`, `agents`, `live`, `daemon`, and more.

- **Agent Daemon**  
  A lightweight background service that collects kernel version, uptime, memory, and load metrics and writes to `/run/eclipta/agent.json`.

- **Live Monitoring (`top`-like)**  
  Beautiful terminal UI to monitor system metrics in real-time (powered by [`tui`](https://github.com/fdehau/tui-rs)).

- **eBPF-Friendly Design**  
  Uses `aya` crate to support attaching custom eBPF programs (e.g. `trace_execve`) to track system calls.

- **Agent-Specific Logs**  
  Fetch logs from any registered agent with one command.

- ⚡ **Pure Rust Stack**  
  From CLI to daemon — built in Rust for performance, safety, and zero dependencies on Python or Node.

---

## Built With

-  [Rust](https://www.rust-lang.org/)
-  [aya](https://github.com/aya-rs/aya) – eBPF for Rust
-  [tui](https://github.com/fdehau/tui-rs) – Terminal UI framework
-  [sysinfo](https://docs.rs/sysinfo) – System stats
-  [clap](https://docs.rs/clap) – Command-line interface
-  [chrono](https://docs.rs/chrono) – Timestamps and formatting

---
