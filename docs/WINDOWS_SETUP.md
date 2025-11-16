# Windows Setup Guide

## Running SIA on Windows

The SIA agent uses Unix domain sockets for IPC, which are not natively supported on Windows. You have a few options:

### Option 1: Use WSL (Recommended)

If you have Windows Subsystem for Linux (WSL) installed:

1. **Start the agent in WSL:**
   ```bash
   wsl
   cd /mnt/g/Git/sia-proto
   cargo run --bin sia-agent
   ```

2. **Use the CLI from Windows:**
   ```bash
   # Set the socket path to the WSL socket
   $env:SIA_SOCKET = "/tmp/sia.sock"
   node cli-ts/dist/index.js status
   ```

   Or from PowerShell:
   ```powershell
   $env:SIA_SOCKET = "\\wsl$\Ubuntu\tmp\sia.sock"  # Adjust for your WSL distro
   node cli-ts/dist/index.js status
   ```

### Option 2: Use Git Bash or MSYS2

If you have Git Bash or MSYS2 installed, you can run both the agent and CLI from there:

1. **Start the agent:**
   ```bash
   cargo run --bin sia-agent
   ```

2. **Use the CLI:**
   ```bash
   node cli-ts/dist/index.js status
   ```

### Option 3: Set Custom Socket Path

You can specify a custom socket path using the `SIA_SOCKET` environment variable:

**PowerShell:**
```powershell
$env:SIA_SOCKET = "C:\path\to\sia.sock"
node cli-ts/dist/index.js status
```

**Command Prompt:**
```cmd
set SIA_SOCKET=C:\path\to\sia.sock
node cli-ts/dist/index.js status
```

**Linux/Mac:**
```bash
export SIA_SOCKET=/custom/path/sia.sock
node cli-ts/dist/index.js status
```

### Checking if the Agent is Running

To check if the agent is running, look for the socket file:

**In WSL:**
```bash
ls -la /tmp/sia.sock
```

**In Git Bash:**
```bash
ls -la /tmp/sia.sock
```

If the socket file exists, the agent is running.

### Troubleshooting

1. **"No SIA socket found" error:**
   - Make sure the agent is running
   - Check the socket path in `config/default.toml` (default: `/tmp/sia.sock`)
   - Set `SIA_SOCKET` environment variable to match the agent's socket path

2. **Connection timeout:**
   - Verify the agent is actually running
   - Check that the socket path is correct
   - On Windows, ensure you're using WSL or a Unix-compatible environment

3. **Permission errors:**
   - Make sure the socket file has proper permissions
   - On Unix systems, the socket should be readable/writable by your user

### Future: Windows Named Pipe Support

Native Windows named pipe support is planned for a future release. This will allow running the agent natively on Windows without WSL.

