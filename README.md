# SIA — System Insight Agent (prototype)

**Status**: ✅ MVP Complete and Working!

This repository contains a working prototype of the local-first System Insight Agent with CPU/memory monitoring, event analysis, LLM integration (Ollama), and a beautiful CLI.

## 🚀 Quick Start

```bash
# Terminal 1: Start the agent
cd /mnt/g/Git/sia-proto
RUST_LOG=info cargo run -p sia-agent

# Terminal 2: Query status
cargo run -p sia-cli -- status
cargo run -p sia-cli -- list
```

See **[QUICKSTART.md](QUICKSTART.md)** for detailed setup and usage.

## ✨ Features

- ✅ **CPU & Memory Monitoring** - Detects high usage and generates events
- ✅ **Event Analysis** - Processes and stores events to SQLite
- ✅ **LLM Integration** - Optional Ollama support for AI-powered suggestions
- ✅ **IPC Server** - JSON-RPC protocol over Unix sockets
- ✅ **Beautiful CLI** - Formatted status, list, and event details
- ✅ **Configuration** - TOML-based with sane defaults
- ✅ **Logging** - Structured logging with env_logger

## Structure
- `common/` - Shared types and logic (events, services, grants)
- `agent/` - Background service for system monitoring
- `cli/` - Command line interface tool
- `sql/schema.sql` - Database schema
- `config/default.toml` - Default configuration

## Prerequisites
- Rust toolchain (1.70+)
- SQLite3 development libraries (for sqlx)

## Quickstart (Linux/macOS)
1. Install Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Build the workspace: `cargo build --workspace`
3. Start the agent: `RUST_LOG=info cargo run -p sia-agent`
4. In another shell, test the CLI: `cargo run -p sia-cli -- status`

## Dependencies
The project uses:
- **serde/serde_json** - Serialization/deserialization
- **sqlx** - Async SQL database access (SQLite)
- **tokio** - Async runtime
- **sysinfo** - System information collection
- **clap** - Command-line argument parsing
- **anyhow** - Error handling

## Development Notes
- The project uses Rust edition 2021
- SQLite is used for local data storage
- System metrics collection runs every 5 seconds
- All database queries use runtime binding (no compile-time checking)

## Troubleshooting
If you encounter build issues:
- Ensure all dependencies are properly specified in `Cargo.toml` files
- Check that feature flags are enabled (e.g., `clap` needs `derive` feature)
- For SQLx errors, ensure queries use runtime binding with `.bind()` methods

> This scaffold is a starting point — replace the TODO placeholders and flesh out collectors, analyzer, LLM adapters, VFS mapping, and tools.