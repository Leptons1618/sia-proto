# SIA Minimal Working Prototype (MVP)

**Goal**: Create a simple but complete working prototype that demonstrates the core system insight agent functionality.

## Core Features (Must-Have)

### 1. ✅ Configuration Loader
- [x] Load from `config/default.toml`
- [x] Parse agent, ipc, llm, storage sections
- [x] Use serde/toml crate

### 2. ✅ Enhanced Collectors
- [x] CPU usage monitoring (every 5s)
- [x] Memory monitoring (system + top processes)
- [x] Send events to channel
- [x] Generate events when thresholds exceeded

### 3. ✅ Event Analyzer
- [x] Receive events from collector channel
- [x] Simple rule-based analysis (threshold checks)
- [x] Send critical events to LLM for suggestions
- [x] Store analyzed events to database

### 4. ✅ LLM Integration (Ollama)
- [x] Connect to Windows Ollama via `ollama.exe` or HTTP
- [x] Send event context to LLM
- [x] Parse LLM suggestions
- [x] Handle errors gracefully (fallback to rule-based)

### 5. ✅ Storage Layer
- [x] Insert events with all fields
- [x] Query recent events (last N)
- [x] Query by severity
- [x] Update event status

### 6. ✅ IPC Server
- [x] Accept Unix socket connections
- [x] JSON-based protocol
- [x] Handle status requests
- [x] Handle list requests (with filters)
- [x] Handle show request (by event_id)

### 7. ✅ CLI Client
- [x] Connect to Unix socket
- [x] `status` - Show agent status + metrics
- [x] `list` - Show recent events
- [x] `show <id>` - Show event details
- [x] Pretty formatted output

## Architecture

```
┌─────────────┐
│ Collectors  │ (CPU, Memory)
└──────┬──────┘
       │ Events via channel
       ↓
┌─────────────┐     ┌─────────────┐
│  Analyzer   │────→│   Ollama    │ (Windows via HTTP/exe)
└──────┬──────┘     └─────────────┘
       │ Analyzed Events
       ↓
┌─────────────┐
│   Storage   │ (SQLite)
└──────┬──────┘
       │
       ↓
┌─────────────┐     ┌─────────────┐
│ IPC Server  │←────│  CLI Client │
└─────────────┘     └─────────────┘
```

## Implementation Plan

### Phase 1: Foundation (Day 1)
1. **Config loader** - Parse TOML and pass to components
2. **Event channel** - tokio::sync::mpsc for collector→analyzer
3. **Storage queries** - get_events(), get_event_by_id()

### Phase 2: Data Collection (Day 1-2)
4. **Enhanced CPU collector** - Detect high CPU, send to channel
5. **Memory collector** - Track memory, detect pressure
6. **Event generation** - Create proper Event structs

### Phase 3: Analysis (Day 2)
7. **Analyzer loop** - Receive events, apply rules
8. **Simple rules** - CPU > 80%, Memory > 85%
9. **Store to DB** - Save all events

### Phase 4: LLM Integration (Day 2-3)
10. **Ollama client** - HTTP client or exec `ollama.exe`
11. **WSL→Windows bridge** - Access via `/mnt/c/` or localhost
12. **Prompt engineering** - Simple event analysis prompt
13. **Response parsing** - Extract suggestions

### Phase 5: IPC (Day 3)
14. **IPC protocol** - JSON request/response
15. **Request handlers** - status, list, show
16. **Connection handling** - Accept clients, read/write JSON

### Phase 6: CLI (Day 3)
17. **Socket client** - Connect to /tmp/sia.sock
18. **Command implementations** - status, list, show
19. **Pretty output** - Format tables/colors

## Simplified Event Rules

```rust
// CPU Rule: If CPU > 80% for 2+ samples → Warning
// CPU Rule: If CPU > 95% → Critical

// Memory Rule: If Memory > 85% → Warning
// Memory Rule: If Memory > 95% → Critical

// Analyzer decides: Critical events → Send to LLM
```

## Ollama Access from WSL

### Option 1: HTTP (Recommended)
```toml
[llm]
ollama_url = "http://localhost:11434"  # Windows host accessible from WSL
model = "llama3.2:3b"  # or whatever model you have
```

### Option 2: Execute Windows Binary
```rust
// Run: /mnt/c/Users/YourUser/AppData/Local/Programs/Ollama/ollama.exe
// Or: cmd.exe /c ollama run llama3.2 "prompt here"
```

We'll use **Option 1 (HTTP)** as it's cleaner and faster.

## Sample Outputs

### CLI: `sia-cli status`
```
SIA Agent Status
================
Uptime: 5m 32s
Status: Running

Collectors:
  CPU:    ✓ Active (5s interval)
  Memory: ✓ Active (5s interval)

Events (last hour):
  Critical: 2
  Warning:  5
  Info:     127

Storage:
  Database: ./sia.db (2.4 MB)
  Events:   134 total
```

### CLI: `sia-cli list`
```
Recent Events
=============
ID       Time     Severity  Type          Description
-------- -------- --------- ------------- ---------------------------
evt_001  12:34:01 CRITICAL  cpu_high      CPU usage at 98% (core 3)
evt_002  12:33:45 WARNING   memory_high   Memory usage at 87%
evt_003  12:33:20 INFO      process_new   Process started: firefox
```

### CLI: `sia-cli show evt_001`
```
Event Details
=============
ID:        evt_001
Time:      2025-11-14 12:34:01
Severity:  CRITICAL
Type:      cpu_high
Service:   system

Entity:
  cpu_usage: 98%
  core: 3
  process: chrome (pid: 1234)

Evidence:
  sustained_high: true
  duration: 45s
  avg_5min: 92%

Suggestion (LLM):
  The CPU spike is caused by Chrome process (PID 1234).
  Recommendation:
  1. Check Chrome task manager (Shift+Esc) for heavy tabs
  2. Consider closing unused tabs
  3. Update Chrome if outdated
  4. Monitor for crypto-mining scripts

Status: open
```

## Technology Stack

- **Config**: `serde` + `toml` crate
- **Channels**: `tokio::sync::mpsc`
- **IPC**: `tokio::net::UnixListener` + `serde_json`
- **LLM**: `reqwest` for HTTP to Ollama
- **Storage**: `sqlx` with SQLite
- **CLI**: `clap` + `tokio::net::UnixStream`
- **Logging**: Replace `println!` with `log` or `tracing`

## Success Criteria

✅ Agent runs continuously, collecting metrics every 5s
✅ High CPU/Memory events detected and stored
✅ Critical events sent to Ollama for AI suggestions
✅ CLI can query agent status and events
✅ All components communicate via channels/IPC
✅ Data persists in SQLite database
✅ Works in WSL with Windows Ollama

## Out of Scope (For Later)

❌ Process collector (CPU/Memory only for MVP)
❌ Disk/Network collectors
❌ Log file monitoring
❌ Complex rule engine (just threshold checks)
❌ VFS/FUSE
❌ Web UI
❌ OAuth/Service discovery
❌ Multi-agent support
❌ Auto-remediation tools

## Next Steps

1. **Start with Phase 1** - Config + Channel + Storage queries
2. **Test each phase** before moving to next
3. **Keep it simple** - Hardcode values where needed
4. **Iterate quickly** - Working prototype > perfect code

Ready to implement? 🚀
