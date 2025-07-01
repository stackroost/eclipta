# eclipta CLI — Self-Hosted Observability Platform

**eclipta** is a lightweight, modular CLI tool for managing, monitoring, and observing Linux agents across your infrastructure. Built in Rust, eclipta enables DevOps, sysadmins, and SREs to inspect system health, manage agent lifecycles, and capture real-time performance metrics — entirely self-hosted.

## Features

- Agent lifecycle control (`load`, `unload`, `restart`, `kill`, `update`)
- Live system metrics (`cpu`, `memory`, `disk`, `network`, `processes`)
- Dynamic agent discovery (`/run/eclipta/*.json`)
- Snapshot syncing to `/etc/eclipta/agents/snapshot.json`
- Configuration management with safe JSON storage
- Realtime monitoring terminal UI (`monitor`, `live`)
- Alert handling and health summaries
- Zero external dependencies — Rust-native

## Installation

### Build from source:
```bash
git clone https://github.com/yourorg/eclipta.git
cd eclipta/cli
cargo build --release
sudo cp target/release/eclipta /usr/local/bin/eclipta
```

## Usage

```bash
eclipta <command> [options]
```

### Common Commands:

| Command           | Description                                  |
|-------------------|----------------------------------------------|
| `load`            | Load/start an agent binary                   |
| `unload`          | Gracefully unload agent                      |
| `restart-agent`   | Restart agent process                        |
| `kill-agent`      | Forcefully kill agent                        |
| `update-agent`    | Replace agent binary with updated version    |
| `monitor`         | Interactive terminal UI of all agents       |
| `live`            | Stream real-time agent logs + stats          |
| `logs`            | View system or agent logs                    |
| `agent-logs`      | Tail logs from a specific agent              |
| `watch-cpu`       | Monitor CPU usage of an agent                |
| `alerts`          | List all agents currently in alert state     |
| `agents`          | Show all detected agents                     |
| `inspect-agent`   | Print detailed stats of a specific agent     |
| `inspect`         | Inspect eclipta CLI environment              |
| `ping-all`        | Check if all agents are alive/responding     |
| `sync-agents`     | Scan `/run/eclipta` and sync active agents   |
| `config`          | Get/set/list CLI configuration options       |
| `version`         | Show current CLI version                     |
| `welcome`         | Show welcome message and setup hint          |
| `status`          | Show CLI runtime status                      |

## Agent Snapshot Format

Synced to: `/etc/eclipta/agents/snapshot.json`

```json
[
  {
    "id": "agent-001",
    "hostname": "host1",
    "version": "0.2.1",
    "cpu_load": [0.39, 1.09, 1.38],
    "mem_used_mb": 4865,
    "disk_used_mb": 79177,
    "net_rx_kb": 247187,
    "alert": false,
    "last_seen": "2025-07-01T16:38:29Z"
  }
]
```

## Development

To run locally:

```bash
cargo run -- monitor
```

## Roadmap

- [ ] `install-agent` from GitHub Releases
- [ ] Remote API mode (multi-node)
- [ ] TUI dashboard for snapshot view
- [ ] Plugin architecture for collectors

## License

MIT © 2025 Mahesh Bhatiya

## Project Structure

```
/cli/
  src/
    commands/
    utils/
    main.rs         # CLI parser & dispatch
/run/eclipta/       # Live agent metrics (.json)
/etc/eclipta/       # Persistent config + snapshot.json
```

## Contributing

PRs welcome! If you're building tooling for observability or Linux system automation, open an issue or suggest improvements.