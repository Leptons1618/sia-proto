# Installation Guide

This guide covers all installation methods for SIA (System Insight Agent).

## System Installation (Recommended)

The recommended way to install SIA is as a system service that runs automatically.

### Prerequisites

- Linux system with systemd (or WSL)
- Rust toolchain (1.70+)
- Node.js (18+) and npm
- SQLite3 development libraries
- Root/sudo access

### Installation Steps

1. **Install dependencies:**

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install build-essential libsqlite3-dev nodejs npm

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. **Clone and install:**

```bash
git clone <repository-url>
cd sia-proto
sudo ./install.sh
```

The install script will:
- Build the Rust agent binary
- Build the TypeScript CLI
- Install binaries to `/usr/local/bin`
- Create system user `sia`
- Set up systemd service
- Create configuration at `/etc/sia/config.toml`
- Initialize database at `/var/lib/sia/sia.db`

3. **Start the service:**

```bash
sudo systemctl start sia-agent
sudo systemctl enable sia-agent  # Enable on boot
```

4. **Verify installation:**

```bash
sia-cli status
```

### Post-Installation

- **Configuration:** `/etc/sia/config.toml`
- **Database:** `/var/lib/sia/sia.db`
- **Socket:** `/run/sia/sia.sock`
- **Logs:** `sudo journalctl -u sia-agent -f`

## Update

To update SIA to the latest version:

```bash
cd sia-proto
git pull
sudo ./update.sh
```

The update script will:
- Rebuild binaries
- Update installed files
- Restart the service (if it was running)

## Uninstall

To remove SIA from your system:

```bash
sudo ./uninstall.sh
```

You'll be prompted whether to remove configuration and data files.

## AppImage (Portable)

For a portable installation that doesn't require system installation:

1. **Build AppImage:**

```bash
./build-appimage.sh
```

2. **Make executable and run:**

```bash
chmod +x appimage-build/sia-x86_64.AppImage
./appimage-build/sia-x86_64.AppImage
```

The AppImage contains both the agent and CLI, and can be run from anywhere.

## Development Installation

For development without system installation:

1. **Build the agent:**

```bash
cargo build --release -p sia-agent
```

2. **Build the CLI:**

```bash
cd cli-ts
npm install
npm run build
```

3. **Run manually:**

```bash
# Terminal 1: Start agent
RUST_LOG=info ./target/release/sia-agent

# Terminal 2: Use CLI
cd cli-ts
node dist/index.js status
```

## Windows / WSL

SIA works in WSL (Windows Subsystem for Linux):

1. Install WSL2 and Ubuntu
2. Follow the Linux installation steps above
3. The service may not auto-start, but you can run manually:

```bash
sudo -u sia /usr/local/bin/sia-agent
```

Or set up a Windows service wrapper if needed.

## Troubleshooting

### Service won't start

```bash
# Check service status
sudo systemctl status sia-agent

# View logs
sudo journalctl -u sia-agent -n 50

# Check permissions
ls -la /var/lib/sia
ls -la /run/sia
```

### Connection refused

- Ensure the agent is running: `sudo systemctl status sia-agent`
- Check socket exists: `ls -la /run/sia/sia.sock`
- Verify socket path in config matches

### Build errors

- Update Rust: `rustup update`
- Update Node.js: `npm install -g npm@latest`
- Reinstall dependencies: `npm install` in `cli-ts/`

### Permission errors

```bash
# Fix ownership
sudo chown -R sia:sia /var/lib/sia
sudo chown -R sia:sia /run/sia
```

