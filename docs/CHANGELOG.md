# Changelog

All notable changes to the SIA prototype project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

## [0.2.0] - 2025-11-15

### Added - System Installation & Service Management

#### Installation System
- **Install script**: Added `install.sh` for system-wide installation
  - Builds release binaries with optimizations
  - Installs to `/usr/local/bin` for global access
  - Creates systemd service for auto-start
  - Sets up system directories (`/etc/sia`, `/var/lib/sia`, `/run/sia`)
  - Updates config paths for system installation
  - Applies security hardening in systemd unit
- **Uninstall script**: Added `uninstall.sh` for clean removal
  - Stops and disables service
  - Removes binaries and service files
  - Optional config and data cleanup
- **Systemd service**: Full systemd integration
  - Auto-restart on failure
  - Memory limits (200MB) and security hardening
  - Journal logging integration
  - Multi-user target for system boot

#### Configuration Enhancements
- **Environment variable support**: Added `SIA_CONFIG` environment variable
  - Allows custom config path via `SIA_CONFIG=/path/to/config.toml`
  - Falls back to `./config/default.toml` if not set
  - Used by systemd service to load from `/etc/sia/config.toml`

#### Documentation Updates
- **QUICKSTART.md**: Added installation section
  - System-wide installation instructions
  - Systemd service management commands
  - Updated troubleshooting for both dev and production modes
  - Added warning about agent persistence in dev mode
- **TODO.md**: Updated completion status
  - Marked 15+ tasks as complete
  - Reflected current implementation state
  - Updated priorities
- **CHANGELOG.md**: This file, documenting all changes

### Changed
- **Config path handling**: Changed from `&'static str` to `String` for dynamic paths
- **Agent startup**: Now uses environment-based config path

### Fixed
- **Cargo workspace warnings**: Moved `resolver = "2"` to workspace level in root Cargo.toml
- **Connection refused errors**: Documented cause (agent not running) and solutions

## [0.1.0] - 2025-11-14

### Added - MVP Implementation (2025-11-14)

#### Configuration System
- **Configuration loader**: Added `common/src/config.rs` with TOML parsing
- **Config structure**: AgentConfig, IpcConfig, LlmConfig, StorageConfig
- **Default config**: Load from `config/default.toml` with all settings

#### Enhanced Collectors
- **Channel-based architecture**: Events sent via tokio::sync::mpsc channel  
- **CPU monitoring**: Detects high CPU usage (>80% WARNING, >95% CRITICAL)
- **Memory monitoring**: Tracks system and per-process memory (>85% WARNING, >95% CRITICAL)
- **Structured events**: Generate Event objects with entity, evidence, and metadata
- **Top process tracking**: Identifies resource-heavy processes in events
- **Configurable intervals**: Respects cpu_interval from config

#### Event Analyzer
- **Event processing pipeline**: Receive events from collectors via channel
- **LLM integration**: Send CRITICAL events to Ollama for AI suggestions
- **Graceful degradation**: Works without LLM if unavailable
- **Database storage**: Store all analyzed events to SQLite
- **Async processing**: Non-blocking event handling

#### LLM Integration (Ollama)
- **HTTP client**: Connect to Ollama API (`reqwest` based)
- **Connection testing**: Verify Ollama availability at startup
- **Prompt engineering**: Structured prompts for system event analysis
- **Response parsing**: Extract AI suggestions into JSON
- **Error handling**: Graceful fallback if LLM unavailable or fails
- **WSL support**: Can connect to Windows Ollama via host IP

#### Storage Layer Enhancements
- **Query methods**: `get_recent_events(limit)`, `get_event_by_id(id)`
- **Event counts**: `get_event_counts()` for status dashboard
- **Stored event struct**: Proper deserialization from database
- **Parameterized queries**: All queries use `.bind()` for safety

#### IPC Server
- **Unix socket server**: Full implementation with tokio::net::UnixListener
- **JSON-RPC protocol**: Simple request/response format
- **Request handlers**: status, list, show commands
- **Concurrent clients**: Handle multiple CLI connections
- **Uptime tracking**: Track agent start time
- **Event filtering**: Support pagination and queries
- **Error responses**: Structured error messages

#### CLI Client
- **Socket communication**: Connect to agent via Unix socket
- **Beautiful output**: Box-drawing characters for formatted display
- **Status command**: Show uptime, collectors, event counts
- **List command**: Tabular event listing with pagination
- **Show command**: Detailed event view with JSON snapshot
- **Help system**: Full clap-based help and subcommands
- **Timestamp formatting**: Human-readable dates and durations
- **Error handling**: User-friendly error messages

#### Logging & Observability
- **Structured logging**: Using `log` crate with `env_logger`
- **Log levels**: INFO for normal operations, WARN for issues
- **Component logging**: Separate logs for collectors, analyzer, storage, IPC
- **RUST_LOG support**: Configure verbosity via environment variable

#### Documentation
- **MVP_PLAN.md**: Detailed implementation plan and architecture
- **QUICKSTART.md**: Complete guide to running and testing the prototype
- **TODO.md**: Comprehensive roadmap for future development

### Technical Improvements

#### Dependencies
- Added `toml = "0.9"` for configuration parsing
- Added `log = "0.4"` and `env_logger = "0.11"` for logging
- Added `reqwest = "0.11"` for HTTP/Ollama client
- Added `chrono` with serde features for timestamp handling

#### Code Quality
- Proper error handling with `anyhow::Result`
- Async/await throughout for non-blocking I/O
- Type-safe Event structures
- Channel-based architecture for decoupling
- Clone-able Storage for multi-threaded access

### Fixed - 2025-11-14

#### Build System
- **Fixed Cargo.toml syntax error**: Corrected malformed line in root `Cargo.toml` where `codegen-units = 1resolver = "2"` was missing a newline separator
- **Fixed workspace resolver warning**: Moved resolver setting from profile.release to workspace level (note: currently generates warning about unused key)

#### Dependencies
- **Replaced placeholder crate**: Changed `serde_toon` (placeholder) to `serde_json` in `common/Cargo.toml` as per inline comment instructions
- **Added missing clap feature**: Enabled `derive` feature for clap in `cli/Cargo.toml` to support procedural macros

#### Type System
- **Fixed serde_json references**: Updated 4 occurrences in `common/src/types.rs` from incorrect `serde_toon::Value` to `serde_json::Value`:
  - `Service.discovery` field
  - `Event.entity` field
  - `Event.evidence` field
  - `Event.suggestion` field

#### System Information Collection
- **Updated sysinfo API usage**: Changed deprecated `global_processor_info()` to `global_cpu_info()` in `agent/src/collectors.rs`
- **Added missing trait imports**: Imported `CpuExt` and `PidExt` traits in `agent/src/collectors.rs` to access CPU and process methods

#### Database Layer
- **Fixed SQLx compile-time checking**: Replaced `sqlx::query!` macro with runtime `sqlx::query()` in `agent/src/storage.rs`
- **Added manual parameter binding**: Converted `insert_event()` method to use `.bind()` for all 6 parameters to avoid DATABASE_URL requirement

### Status

#### ✅ Fully Working
- Agent runs continuously, collecting metrics every 5s
- High CPU/Memory events detected and stored
- Critical events can be sent to Ollama for AI suggestions (if configured)
- CLI can query agent status and events via IPC
- All components communicate via channels/IPC
- Data persists in SQLite database
- Works in WSL with optional Windows Ollama integration

#### 🎯 MVP Goals Achieved
1. ✅ Configuration management
2. ✅ Enhanced collectors (CPU, Memory)
3. ✅ Event analyzer with channel pipeline
4. ✅ LLM integration (Ollama)
5. ✅ Storage layer with queries
6. ✅ IPC server with JSON protocol
7. ✅ CLI client with pretty output

### Known Issues
- Workspace resolver warning persists (resolver = "1" vs "2" for edition 2021)
- Ollama connection requires manual configuration for WSL→Windows access
- Some unused import warnings in CLI (cosmetic)
- Database file path must exist (no auto-creation of parent dirs)

---

## Initial Release

### Added
- Basic project structure with workspace organization
- Common types library for shared data structures
- Agent service with system monitoring capabilities
- CLI tool for interacting with the agent
- SQLite-based storage layer
- System information collection using sysinfo crate
- Async runtime with Tokio
- Configuration framework
