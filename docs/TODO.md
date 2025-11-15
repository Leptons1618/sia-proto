# SIA Prototype - TODO List

Generated: 2025-11-14

## High Priority - Core Functionality

### 🔴 Configuration Management
- [x] Implement config loader from `config/default.toml`
- [x] Support config file override via environment variable (SIA_CONFIG)
- [ ] Add config file override support (CLI args)
- [ ] Validate configuration values on startup
- [ ] Add config hot-reload capability
- [ ] Support per-user config files (~/.sia/config.toml)

### 🔴 Storage Layer Enhancement
- [x] Implement query methods (get_events, get_grants, get_audits)
- [x] Implement event status tracking (open status)
- [ ] Add pagination support for event queries
- [ ] Implement event filtering by severity, type, service_id
- [ ] Add event status update methods (open → resolved)
- [ ] Implement grants management (create, revoke, refresh)
- [ ] Add database migration system
- [ ] Implement data retention policies (based on disk_quota)
- [ ] Add storage metrics (size, record counts)

### 🔴 IPC Server Implementation
- [x] Implement actual Unix socket request handling
- [x] Define IPC protocol/message format (JSON-RPC)
- [x] Handle client connections and disconnections
- [x] Implement request routing (status, list, query, etc.)
- [ ] Add authentication/authorization for IPC
- [ ] Windows named pipe support
- [ ] Add connection pooling/limits
- [x] Implement graceful shutdown for active connections

### 🔴 CLI Enhancement
- [x] Connect CLI to agent via IPC socket
- [x] Implement `status` command with real agent data
  - [x] Show uptime
  - [ ] Display memory/CPU usage
  - [x] Show collector status
  - [x] Display event counts by severity
- [x] Implement `list` command with filters
  - [ ] Filter by severity (--severity=error)
  - [ ] Filter by service (--service=xyz)
  - [ ] Filter by time range (--since, --until)
  - [x] Add pagination (limit parameter)
- [x] Add `show <event-id>` command for event details
- [ ] Add `resolve <event-id>` command
- [ ] Add `grants` subcommand for managing service grants
- [ ] Add `config` subcommand to view/modify settings
- [ ] Implement JSON output format (--json flag)
- [ ] Add interactive mode

## Medium Priority - System Monitoring

### 🟡 Collectors Enhancement
- [x] Implement channel-based event pipeline
  - [x] Create mpsc channel for collector → analyzer
  - [x] Implement event ring buffer (10k capacity)
  - [x] Add backpressure handling
- [x] CPU Collector improvements
  - [x] Track per-core usage
  - [x] Detect sustained high CPU (> 80% for 30s)
  - [ ] Calculate moving averages
  - [x] Honor cpu_interval from config
- [x] Memory Collector
  - [x] Track system memory usage
  - [ ] Track swap usage
  - [x] Detect memory pressure events
  - [x] Track per-process memory (top 10)
- [ ] Process Collector
  - [ ] List all running processes
  - [ ] Track new process spawns
  - [ ] Detect suspicious process patterns
  - [ ] Monitor resource-heavy processes
  - [ ] Honor proc_interval from config
- [ ] Disk Collector
  - [ ] Monitor disk usage per mount point
  - [ ] Track I/O rates
  - [ ] Detect disk space warnings (> 85%)
  - [ ] Monitor disk health (SMART data)
- [ ] Network Collector
  - [ ] Track active connections
  - [ ] Monitor bandwidth usage
  - [ ] Detect unusual network activity
  - [ ] Track listening ports
- [ ] Log Collector
  - [ ] Tail system logs (/var/log/syslog, journalctl)
  - [ ] Parse and categorize log entries
  - [ ] Detect error patterns
  - [ ] Integrate with syslog

### 🟡 Analyzer Implementation
- [x] Implement event analyzer stub
  - [x] Receive events from collector channel
  - [x] Apply rule-based analysis
  - [ ] Generate insights from patterns
- [x] Rule Engine (basic threshold-based)
  - [ ] Define rule format (YAML/TOML?)
  - [ ] Implement rule loader
  - [x] Support threshold-based rules
  - [ ] Support pattern-matching rules
  - [ ] Support time-window aggregations
- [ ] Anomaly Detection
  - [ ] Track baseline metrics
  - [ ] Detect deviations from baseline
  - [ ] Implement simple ML models (optional)
- [ ] Event Correlation
  - [ ] Group related events by fingerprint
  - [ ] Detect event storms
  - [ ] Generate summary events
- [ ] Severity Classification
  - [ ] Auto-assign severity levels
  - [ ] Support severity escalation
  - [ ] Implement severity decay

## Medium Priority - LLM Integration

### 🟡 LLM Adapter
- [x] Implement Ollama client
  - [x] Test connection to ollama_url
  - [x] Send prompts and receive responses
  - [x] Handle timeouts and errors
  - [ ] Add retry logic with exponential backoff
- [x] Prompt Engineering
  - [x] Design system prompt for event analysis
  - [x] Create prompt templates for different event types
  - [ ] Implement context window management
  - [ ] Add few-shot examples
- [x] Response Processing
  - [x] Parse LLM responses
  - [x] Extract suggestions and remediation steps
  - [ ] Validate LLM outputs
  - [x] Store suggestions in events table
- [ ] Fallback Support
  - [ ] Support OpenAI API as fallback
  - [ ] Support Anthropic API
  - [ ] Support local transformers (gguf models)
- [ ] Privacy & Security
  - [ ] Sanitize events before sending to LLM
  - [ ] Implement PII detection/redaction
  - [ ] Add opt-in/opt-out for LLM features
  - [ ] Audit LLM interactions

## Low Priority - Advanced Features

### 🟢 Service Management
- [ ] Implement service discovery
  - [ ] Auto-detect running services
  - [ ] Parse service metadata
  - [ ] Store service configs
- [ ] OAuth/Token Management
  - [ ] Implement grant creation workflow
  - [ ] Support token refresh
  - [ ] Handle token expiration
  - [ ] Secure token storage (keyring integration?)
- [ ] Service Scopes
  - [ ] Define scope permissions
  - [ ] Implement scope validation
  - [ ] Support dynamic scope requests

### 🟢 Virtual File System (VFS)
- [ ] Design VFS interface
  - [ ] Define path structure (/system, /services, /events)
  - [ ] Implement FUSE driver (optional)
  - [ ] Support read operations
  - [ ] Support write operations for config
- [ ] VFS Mappings
  - [ ] Map events to /events/<id>
  - [ ] Map services to /services/<name>
  - [ ] Map system metrics to /system/cpu, /system/mem
  - [ ] Map logs to /logs/<source>

### 🟢 Tools & Actions
- [ ] Define tool interface
- [ ] Implement system tools
  - [ ] Restart service
  - [ ] Kill process
  - [ ] Clear cache
  - [ ] Run diagnostic script
- [ ] Implement remediation actions
  - [ ] Auto-remediation with approval
  - [ ] Manual remediation triggers
  - [ ] Rollback support
- [ ] Tool safety
  - [ ] Require confirmation for dangerous ops
  - [ ] Implement dry-run mode
  - [ ] Audit all tool executions

### 🟢 Web UI / Dashboard
- [ ] Design web interface
- [ ] Implement HTTP server
- [ ] Real-time event stream (WebSocket)
- [ ] Event list view with filtering
- [ ] Event detail view
- [ ] System metrics dashboard
- [ ] Service management UI
- [ ] Configuration editor

### 🟢 Notifications
- [ ] Email notifications
- [ ] Slack/Discord webhooks
- [ ] Desktop notifications (Linux/macOS)
- [ ] SMS notifications (Twilio integration)
- [ ] Custom webhook support

### 🟢 Multi-System Support
- [ ] Central aggregation server
- [ ] Agent → server communication
- [ ] Multi-tenant support
- [ ] Fleet management

## Testing & Quality

### 🔵 Testing
- [ ] Unit tests for storage layer
- [ ] Unit tests for collectors
- [ ] Unit tests for analyzer
- [ ] Integration tests for IPC
- [ ] End-to-end CLI tests
- [ ] Load testing for event pipeline
- [ ] Fuzzing for IPC protocol
- [ ] Add CI/CD pipeline (GitHub Actions)

### 🔵 Documentation
- [ ] API documentation (rustdoc)
- [ ] Architecture diagram
- [ ] IPC protocol specification
- [ ] Configuration reference
- [ ] Troubleshooting guide
- [ ] Contributing guidelines
- [ ] Security policy

### 🔵 Performance
- [ ] Profile CPU usage
- [ ] Profile memory usage
- [ ] Optimize hot paths
- [ ] Implement memory budget enforcement
- [ ] Add performance benchmarks
- [ ] Optimize database queries (indexes, prepared statements)

### 🔵 Security
- [ ] Security audit
- [ ] Input validation everywhere
- [ ] SQL injection prevention (verify parameterized queries)
- [ ] Implement rate limiting
- [ ] Add access control
- [ ] Secure IPC authentication
- [ ] Dependency vulnerability scanning

## Infrastructure

### 🟤 Packaging & Distribution
- [x] Create install script
- [ ] Package for major Linux distros (deb, rpm)
- [ ] Homebrew formula (macOS)
- [ ] Windows installer
- [ ] Docker image
- [x] Systemd service unit
- [ ] Launchd plist (macOS)

### 🟤 Logging & Observability
- [x] Structured logging (log + env_logger)
- [x] Log levels (TRACE, DEBUG, INFO, WARN, ERROR)
- [ ] Log rotation
- [ ] Metrics export (Prometheus?)
- [ ] Health check endpoint
- [ ] Self-monitoring (agent monitoring itself)

---

## Quick Wins (Start Here! 🚀)

Good first tasks to get momentum:

1. **Load config from file** - Replace hardcoded values in main.rs
2. **Implement status command** - Return real data via IPC
3. **Add event query methods** - Basic SELECT queries in storage.rs
4. **Create event channel** - Connect collectors → analyzer
5. **Implement memory collector** - Add to collectors.rs
6. **Add unit tests** - Start with storage layer
7. **Create architecture diagram** - Document the design
8. **Add trace logging** - Replace println! with proper logging

## Notes

- **Prototype Status**: Many features have stubs marked with "TODO" or "placeholder"
- **Priority Legend**: 🔴 High | 🟡 Medium | 🟢 Low | 🔵 Quality | 🟤 Ops
- **Memory Budget**: Respect 200MB limit from config
- **Disk Quota**: Respect 500MB limit from config
- **Local-First**: All data stays local, LLM is optional/pluggable
