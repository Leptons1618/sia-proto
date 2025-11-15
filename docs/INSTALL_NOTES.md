# Installation Notes

## Running the Installation

The install script needs to be run with `sudo` to install system files, but it will automatically use your user account to build the Rust binaries (since root doesn't have Rust configured).

```bash
# This is correct - the script handles everything
sudo ./install.sh
```

## What the Script Does

1. Checks if Rust/Cargo is available
2. **Builds as your user** (not root) to use your Rust toolchain
3. **Installs as root** to system directories
4. Sets up systemd service
5. Configures directories and permissions

## Troubleshooting

### "Cargo not found" error

This means you need to install Rust first:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### "No default toolchain" error

Set up the default Rust toolchain:

```bash
rustup default stable
```

### Build errors

Make sure you have development dependencies:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential libsqlite3-dev pkg-config libssl-dev
```

## Manual Installation (Alternative)

If the automatic script doesn't work, you can install manually:

```bash
# 1. Build as your user
cargo build --release --workspace

# 2. Install binaries (needs sudo)
sudo cp target/release/sia-agent /usr/local/bin/
sudo cp target/release/sia-cli /usr/local/bin/
sudo chmod +x /usr/local/bin/sia-agent /usr/local/bin/sia-cli

# 3. Create directories
sudo mkdir -p /etc/sia /var/lib/sia /run/sia

# 4. Copy config
sudo cp config/default.toml /etc/sia/config.toml

# 5. Update config paths
sudo sed -i 's|socket_path = "/tmp/sia.sock"|socket_path = "/run/sia/sia.sock"|g' /etc/sia/config.toml
sudo sed -i 's|db_path = "./sia.db"|db_path = "/var/lib/sia/sia.db"|g' /etc/sia/config.toml

# 6. Create systemd service
sudo tee /etc/systemd/system/sia-agent.service > /dev/null <<EOF2
[Unit]
Description=System Insight Agent
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/sia-agent
Restart=always
RestartSec=10
Environment="RUST_LOG=info"
Environment="SIA_CONFIG=/etc/sia/config.toml"
WorkingDirectory=/var/lib/sia
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF2

# 7. Enable and start
sudo systemctl daemon-reload
sudo systemctl enable sia-agent
sudo systemctl start sia-agent
```

## Verification

Check if everything is working:

```bash
# Check service status
sudo systemctl status sia-agent

# Test CLI
sia-cli status

# View logs
sudo journalctl -u sia-agent -f
```
