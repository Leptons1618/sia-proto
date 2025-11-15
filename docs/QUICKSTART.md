# SIA MVP - Quick Start Guide

## What Was Built

A minimal working prototype of the System Insight Agent with:

✅ **Configuration Loader** - Reads `config/default.toml`
✅ **CPU & Memory Collectors** - Monitor system resources every 5s  
✅ **Event Analyzer** - Processes events and stores to database
✅ **LLM Integration** - Sends critical events to Ollama for AI suggestions
✅ **Storage Layer** - SQLite database with query methods
✅ **IPC Server** - Unix socket JSON-RPC protocol
✅ **CLI Client** - Beautiful formatted output for status, list, show
✅ **Systemd Service** - Run as a system service

## Installation (Recommended)

### System-Wide Installation

Install SIA as a system service that runs automatically:

```bash
# Build and install
sudo ./install.sh

# Start the service
sudo systemctl start sia-agent

# Enable on boot
sudo systemctl enable sia-agent

# Check status
sia-cli status

# View logs
sudo journalctl -u sia-agent -f
```

The installer will:
- Build release binaries
- Install `sia-agent` and `sia-cli` to `/usr/local/bin`
- Create systemd service for auto-start
- Set up config at `/etc/sia/config.toml`
- Configure data directory at `/var/lib/sia`

**Note**: After installation, you can run `sia-cli` from anywhere!

### Uninstall

```bash
sudo ./uninstall.sh
```

## Development Mode (Manual Running)

## Development Mode (Manual Running)

For development, you can run the agent manually without installing:

### Terminal 1: Start the Agent

```bash
cd /mnt/g/Git/sia-proto
rm -f sia.db /tmp/sia.sock  # Clean start
RUST_LOG=info cargo run -p sia-agent
```

You should see:
```
[INFO sia_agent] Starting SIA agent (MVP prototype)
[INFO sia_agent] Config loaded from ./config/default.toml
[INFO sia_agent] Storage initialized at ./sia.db
[INFO sia_agent] LLM not available, continuing without AI suggestions
[INFO sia_agent] Collectors started
[INFO sia_agent] Analyzer started
[INFO sia_agent] IPC server started on /tmp/sia.sock
[INFO sia_agent] SIA agent is running
```

**Note**: If you see "LLM not available", that's OK! The agent works without it.

**⚠️ Important**: Keep this terminal running! The agent must be running for the CLI to work. If you close this terminal or switch directories, the agent will stop and the CLI will show "Connection refused".

For persistent operation, use the system installation method above instead.

### Terminal 2: Use the CLI

```bash
# Check agent status
cargo run -p sia-cli -- status

# List recent events
cargo run -p sia-cli -- list

# Show specific event details
cargo run -p sia-cli -- show <event-id>

# Get help
cargo run -p sia-cli -- --help
```

## Testing Event Generation

The collectors will automatically generate events when:
- **CPU > 80%** for 2+ consecutive checks → WARNING
- **CPU > 95%** → CRITICAL
- **Memory > 85%** → WARNING  
- **Memory > 95%** → CRITICAL

To trigger events, you can:

```bash
# CPU stress (install stress-ng if needed)
sudo apt-get install stress-ng
stress-ng --cpu 4 --timeout 30s

# Or use a simple CPU burner
yes > /dev/null &  # Run 4 of these
# Kill with: pkill yes
```

Within 5-10 seconds, the collectors should detect high CPU and create events.

## Ollama Integration (Optional)

### Setup Ollama on Windows

1. Ensure Ollama is running on Windows
2. Make it accessible from WSL:

**Option A: Expose Ollama to network**
```powershell
# In Windows PowerShell (as Admin)
setx OLLAMA_HOST "0.0.0.0:11434"
# Restart Ollama service
```

**Option B: Use Windows host IP**
```bash
# Find Windows host IP from WSL
cat /etc/resolv.conf | grep nameserver

# Update config/default.toml
[llm]
ollama_url = "http://10.255.255.254:11434"  # Use the IP from above
model = "llama3.2"  # or whatever model you have installed
```

**Option C: Port forwarding (if Ollama only listens on localhost)**
```powershell
# In Windows PowerShell (as Admin)
netsh interface portproxy add v4tov4 listenport=11434 listenaddress=0.0.0.0 connectport=11434 connectaddress=127.0.0.1
```

### Test Ollama Connection

```bash
# From WSL
curl http://localhost:11434/api/tags
# or
curl http://10.255.255.254:11434/api/tags
```

If this returns JSON with your models, update `config/default.toml` with the correct URL and restart the agent.

## Expected Output

### `cargo run -p sia-cli -- status`

```
╔═══════════════════════════════════════════════════════════════╗
║                    SIA Agent Status                           ║
╠═══════════════════════════════════════════════════════════════╣
║ Uptime:     2m 34s                                            ║
║ Status:     running                                           ║
╠═══════════════════════════════════════════════════════════════╣
║ Collectors:                                                   ║
║   CPU:      ✓ active                                          ║
║   Memory:   ✓ active                                          ║
╠═══════════════════════════════════════════════════════════════╣
║ Events (open):                                                ║
║   Critical: 2                                                 ║
║   Warning:  5                                                 ║
║   Info:     0                                                 ║
╚═══════════════════════════════════════════════════════════════╝
```

### `cargo run -p sia-cli -- list`

```
┌──────────────────────────┬────────────────────────┬──────────┬───────────────┬────────┐
│ Event ID                 │ Timestamp              │ Severity │ Type          │ Status │
├──────────────────────────┼────────────────────────┼──────────┼───────────────┼────────┤
│ cpu_1731612345678        │ 2025-11-14 17:05:45    │ CRITICAL │ cpu_high      │ open   │
│ mem_1731612340123        │ 2025-11-14 17:05:40    │ WARNING  │ memory_high   │ open   │
└──────────────────────────┴────────────────────────┴──────────┴───────────────┴────────┘
```

### `cargo run -p sia-cli -- show cpu_1731612345678`

```
╔═══════════════════════════════════════════════════════════════╗
║                      Event Details                            ║
╠═══════════════════════════════════════════════════════════════╣
║ ID:         cpu_1731612345678                                 ║
║ Time:       2025-11-14 17:05:45                               ║
║ Severity:   CRITICAL                                          ║
║ Type:       cpu_high                                          ║
║ Service:    system                                            ║
║ Status:     open                                              ║
╠═══════════════════════════════════════════════════════════════╣
║ Snapshot Data:                                                ║
║ {                                                             ║
║   "cpu_usage": 98.5,                                          ║
║   "type": "system_cpu",                                       ║
║   "top_process": {                                            ║
║     "name": "stress-ng-cpu",                                  ║
║     "pid": 12345,                                             ║
║     "cpu": 98.2                                               ║
║   }                                                           ║
║ }                                                             ║
╚═══════════════════════════════════════════════════════════════╝
```

## Architecture Flow

```
Collectors (CPU/Mem) 
    ↓ (events via channel)
Analyzer
    ├→ LLM (if available, for CRITICAL events)
    └→ Storage (SQLite)
         ↓
    IPC Server (/tmp/sia.sock)
         ↓
    CLI Client (status/list/show)
```

## Troubleshooting

**"Error: Connection refused"**
- **System installation**: Check if service is running: `sudo systemctl status sia-agent`
- **Dev mode**: Agent not running. Start it first in another terminal.
- **Wrong socket**: Check `socket_path` in config matches where agent is running

**"unable to open database file"**
- **System installation**: Check permissions on `/var/lib/sia/`
- **Dev mode**: Run from project root: `cd /mnt/g/Git/sia-proto`
- Or create directory for db: `mkdir -p $(dirname ./sia.db)`

**"LLM not available"**
- This is OK! The agent works without LLM
- See "Ollama Integration" section to enable it

**No events showing up**
- Wait 5-10 seconds for collectors to run
- Generate load (CPU/memory stress)
- Check agent logs for collector activity

## What's Working

- ✅ Agent runs continuously (as systemd service or manual)
- ✅ CLI accessible from anywhere (after system install)
- ✅ Collectors monitor CPU and memory every 5 seconds
- ✅ Events are generated when thresholds exceeded
- ✅ Events are stored in SQLite database
- ✅ IPC server accepts connections
- ✅ CLI can query status and events
- ✅ Pretty formatted output
- ✅ LLM integration (when Ollama is accessible)
- ✅ Auto-start on system boot (with systemd)

## Next Steps

See `TODO.md` for future enhancements:
- Process collector
- Disk/Network collectors
- Rule engine improvements
- Web UI
- Auto-remediation
- And much more!

## Files Modified/Created

**New Files:**
- `common/src/config.rs` - Configuration loader
- `agent/src/llm.rs` - Ollama LLM client
- `MVP_PLAN.md` - This guide

**Modified Files:**
- `agent/src/collectors.rs` - Enhanced with event generation
- `agent/src/analyzer.rs` - Event processing with LLM integration
- `agent/src/storage.rs` - Query methods added
- `agent/src/ipc.rs` - Full JSON-RPC implementation
- `agent/src/main.rs` - Wired everything together
- `cli/src/main.rs` - Beautiful CLI with socket communication
- `config/default.toml` - Updated Ollama settings

**Dependencies Added:**
- `toml` - Config parsing
- `log` + `env_logger` - Logging
- `reqwest` - HTTP client for Ollama
- `chrono` - Timestamp handling

Enjoy your working prototype! 🚀
