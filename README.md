# SIA — System Insight Agent

A local-first system monitoring and analysis tool with CPU/memory monitoring, event analysis, LLM integration (Ollama), and a beautiful CLI.

## Features

- **CPU & Memory Monitoring** - Detects high usage and generates events
- **Event Analysis** - Processes and stores events to SQLite
- **LLM Integration** - Optional Ollama support for AI-powered suggestions
- **IPC Server** - JSON-RPC protocol over Unix sockets
- **Beautiful CLI** - Formatted status, list, and event details
- **Systemd Service** - Run as a persistent system service
- **Configuration** - TOML-based with environment variable support
- **Logging** - Structured logging with journal integration

## Quick Start

### System Installation (Recommended)

Install SIA as a system service that runs automatically:

```bash
# Build and install (requires sudo)
sudo ./install.sh

# Start the service
sudo systemctl start sia-agent

# Enable on boot
sudo systemctl enable sia-agent

# Use the CLI from anywhere
sia-cli status
sia-cli list
sia-cli show <event-id>

# View logs
sudo journalctl -u sia-agent -f
```

After installation:
- Binaries: `/usr/local/bin/sia-agent`, `/usr/local/bin/sia-cli`
- Config: `/etc/sia/config.toml`
- Data: `/var/lib/sia/sia.db`
- Socket: `/run/sia/sia.sock`

### Uninstall

```bash
sudo ./uninstall.sh
```

### Development Mode

For development without system installation:

```bash
# Terminal 1: Start the agent
RUST_LOG=info cargo run -p sia-agent

# Terminal 2: Use the CLI
cargo run -p sia-cli -- status
cargo run -p sia-cli -- list
```

**Note**: In dev mode, keep the agent terminal running. Closing it will cause "Connection refused" errors.

## Documentation

- **[QUICKSTART.md](docs/QUICKSTART.md)** - Complete guide to installation and usage
- **[TODO.md](docs/TODO.md)** - Development roadmap and task list
- **[CHANGELOG.md](docs/CHANGELOG.md)** - Version history and changes
- **[MVP_PLAN.md](docs/MVP_PLAN.md)** - Original MVP implementation plan

## Structure

```
sia-proto/
├── common/         # Shared types and configuration
├── agent/          # Background monitoring service
├── cli/            # Command line interface
├── config/         # Configuration files
├── sql/            # Database schema
├── docs/           # Documentation
├── install.sh      # System installation script
└── uninstall.sh    # Uninstallation script
```

## Prerequisites

- Rust toolchain (1.70+)
- SQLite3 development libraries
- Linux with systemd (for service installation)

```bash
# Ubuntu/Debian
sudo apt-get install build-essential libsqlite3-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Building from Source

```bash
# Clone the repository
git clone <repo-url>
cd sia-proto

# Build workspace
cargo build --workspace

# Or build release
cargo build --release --workspace
```

## Configuration

Edit `/etc/sia/config.toml` (system install) or `config/default.toml` (dev mode):

```toml
[agent]
memory_budget = 209715200     # 200MB
disk_quota = 524288000        # 500MB
cpu_interval = 5              # seconds
proc_interval = 10            # seconds

[ipc]
socket_path = "/run/sia/sia.sock"

[llm]
ollama_url = "http://localhost:11434"
model = "llama3.2"

[storage]
db_path = "/var/lib/sia/sia.db"
```

Environment variable: `SIA_CONFIG=/path/to/config.toml`

## Troubleshooting

**"Connection refused" error:**
- System install: Check service status with `sudo systemctl status sia-agent`
- Dev mode: Ensure agent is running in another terminal

**Service won't start:**
- Check logs: `sudo journalctl -u sia-agent -n 50`
- Verify config: `cat /etc/sia/config.toml`
- Check permissions on `/var/lib/sia` and `/run/sia`

**Build errors:**
- Ensure SQLite dev libraries are installed
- Update Rust: `rustup update`

## Dependencies

The project uses:
- **tokio** - Async runtime
- **serde/serde_json** - Serialization
- **sqlx** - Async SQLite database
- **sysinfo** - System information collection
- **clap** - CLI argument parsing
- **anyhow** - Error handling
- **reqwest** - HTTP client for Ollama
- **chrono** - Timestamp handling
- **log/env_logger** - Logging

## What's Working

Current MVP features:
- Agent runs as systemd service or manually
- CPU and memory monitoring (5s interval)
- Event generation on threshold breaches
- SQLite storage with queries
- Unix socket IPC with JSON-RPC
- CLI with status/list/show commands
- Ollama LLM integration (optional)
- System-wide installation
- Auto-start on boot

## License

[Add your license here]

## Contributing

See [TODO.md](docs/TODO.md) for planned features and tasks.