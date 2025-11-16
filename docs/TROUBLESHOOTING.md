# Troubleshooting Guide

## Common Issues

### "No SIA socket found. Is the agent running?"

This error means the SIA agent service is not running. Here's how to fix it:

#### For System Installation (systemd)

1. **Check if the service exists:**
   ```bash
   sudo systemctl status sia-agent
   ```

2. **Start the service:**
   ```bash
   sudo systemctl start sia-agent
   ```

3. **Enable it to start on boot (optional):**
   ```bash
   sudo systemctl enable sia-agent
   ```

4. **Verify it's running:**
   ```bash
   sudo systemctl status sia-agent
   sia-cli status
   ```

5. **Check logs if it's not starting:**
   ```bash
   sudo journalctl -u sia-agent -n 50
   ```

#### For Development Mode

If you're running the agent manually (not as a service):

1. **Start the agent in a terminal:**
   ```bash
   cargo run -p sia-agent
   ```

2. **Keep that terminal open** - closing it will stop the agent

3. **In another terminal, use the CLI:**
   ```bash
   node cli-ts/dist/index.js status
   ```

### Service Fails to Start

If the service fails to start, check the logs:

```bash
sudo journalctl -u sia-agent -n 50 --no-pager
```

Common issues:
- **Permission errors**: Make sure the `sia` user exists and has proper permissions
- **Config file missing**: Check if `/etc/sia/config.toml` exists
- **Socket directory**: Make sure `/run/sia` directory exists and is writable

### Socket Path Issues

The CLI checks these paths in order:
1. `/run/sia/sia.sock` (system installation)
2. `/tmp/sia.sock` (development mode)
3. Environment variable `SIA_SOCKET` (custom path)

To use a custom socket path:
```bash
export SIA_SOCKET=/custom/path/sia.sock
sia-cli status
```

### Check Socket File Exists

```bash
# For system installation
ls -la /run/sia/sia.sock

# For development mode
ls -la /tmp/sia.sock
```

If the socket file doesn't exist, the agent is not running.

### Service Not Found

If `systemctl status sia-agent` says "Unit sia-agent.service could not be found":

1. **Reinstall the service:**
   ```bash
   sudo ./install.sh
   ```

2. **Or manually create the service file** (see install.sh for the service file content)

### Permission Denied

If you get permission errors:

1. **Check socket permissions:**
   ```bash
   ls -la /run/sia/sia.sock
   ```

2. **Fix permissions if needed:**
   ```bash
   sudo chmod 666 /run/sia/sia.sock
   ```

3. **Or run CLI with sudo** (not recommended, but works):
   ```bash
   sudo sia-cli status
   ```

### Connection Timeout

If you get connection timeout errors:

1. **Verify the agent is actually running:**
   ```bash
   ps aux | grep sia-agent
   ```

2. **Check if the socket is listening:**
   ```bash
   sudo netstat -lx | grep sia
   # or
   sudo ss -lx | grep sia
   ```

3. **Check firewall rules** (unlikely, but possible)

### After Update

After running `update.sh`:

1. **The service may have been stopped** - restart it:
   ```bash
   sudo systemctl start sia-agent
   ```

2. **If the service wasn't running before update**, you need to start it manually:
   ```bash
   sudo systemctl start sia-agent
   ```

