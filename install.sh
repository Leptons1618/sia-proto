#!/bin/bash
set -e

# SIA Installation Script
# Builds binaries and installs them system-wide with systemd service

INSTALL_DIR="/usr/local/bin"
SERVICE_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/sia"
DATA_DIR="/var/lib/sia"
SOCKET_DIR="/run/sia"

echo "🚀 Installing SIA (System Insight Agent)..."

# Check if running as root/sudo
if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run with sudo: sudo ./install.sh"
    exit 1
fi

# Get the original user (who ran sudo)
ORIGINAL_USER="${SUDO_USER:-$USER}"
ORIGINAL_HOME=$(eval echo ~$ORIGINAL_USER)

# Get absolute path to project directory (works better in WSL)
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build release binaries as the original user (not root)
echo "📦 Building release binaries..."
if [ -n "$SUDO_USER" ]; then
    # Running under sudo - build as the original user
    # Use sudo -u instead of su - for better WSL compatibility
    sudo -u "$SUDO_USER" bash -c "cd '$PROJECT_DIR' && cargo build --release --workspace"
else
    # Running as root directly
    cd "$PROJECT_DIR"
    cargo build --release --workspace
fi

# Create directories
echo "📁 Creating directories..."
mkdir -p "$CONFIG_DIR"
mkdir -p "$DATA_DIR"
mkdir -p "$SOCKET_DIR"

# Create sia user if it doesn't exist
if ! id -u sia >/dev/null 2>&1; then
    echo "👤 Creating sia system user..."
    useradd --system --no-create-home --shell /bin/false sia
fi

# Set ownership and permissions
chown -R sia:sia "$DATA_DIR"
chown -R sia:sia "$SOCKET_DIR"
chmod 755 "$DATA_DIR"
chmod 755 "$SOCKET_DIR"

# Install binaries
echo "🔧 Installing binaries to $INSTALL_DIR..."
cp target/release/sia-agent "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/sia-agent"

# Install TypeScript CLI
echo "🔧 Installing TypeScript CLI..."
cd "$PROJECT_DIR/cli-ts"
if [ -n "$SUDO_USER" ]; then
    sudo -u "$SUDO_USER" bash -c "cd '$PROJECT_DIR/cli-ts' && npm install && npm run build"
else
    npm install && npm run build
fi

# Create CLI symlink or wrapper script
if [ ! -f "$INSTALL_DIR/sia-cli" ]; then
    cat > "$INSTALL_DIR/sia-cli" <<'EOF'
#!/bin/bash
cd "$(dirname "$0")/../lib/sia/cli-ts" 2>/dev/null || cd "/usr/lib/sia/cli-ts" 2>/dev/null || {
    echo "Error: CLI not found. Please reinstall SIA."
    exit 1
}
node dist/index.js "$@"
EOF
    chmod +x "$INSTALL_DIR/sia-cli"
    
    # Copy CLI files to system location
    mkdir -p /usr/lib/sia/cli-ts
    cp -r "$PROJECT_DIR/cli-ts/dist" /usr/lib/sia/cli-ts/
    cp "$PROJECT_DIR/cli-ts/package.json" /usr/lib/sia/cli-ts/
    # Install production dependencies only
    cd /usr/lib/sia/cli-ts
    npm install --production --no-save
fi

# Copy default config if doesn't exist
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    echo "⚙️  Installing default configuration..."
    cp config/default.toml "$CONFIG_DIR/config.toml"
else
    echo "ℹ️  Configuration already exists at $CONFIG_DIR/config.toml"
fi

# Update config paths for system installation
echo "🔄 Updating configuration paths..."
sed -i "s|socket_path = \"/tmp/sia.sock\"|socket_path = \"/run/sia/sia.sock\"|g" "$CONFIG_DIR/config.toml"
sed -i "s|db_path = \"./sia.db\"|db_path = \"/var/lib/sia/sia.db\"|g" "$CONFIG_DIR/config.toml"

# Initialize database with schema
if [ ! -f "$DATA_DIR/sia.db" ]; then
    echo "💾 Initializing database..."
    SCHEMA_FILE="$PROJECT_DIR/sql/schema.sql"
    cat "$SCHEMA_FILE" | sudo -u sia sqlite3 "$DATA_DIR/sia.db"
fi

# Install systemd service (if systemd is available)
if command -v systemctl &> /dev/null; then
    echo "🔌 Installing systemd service..."
    cat > "$SERVICE_DIR/sia-agent.service" <<EOF
[Unit]
Description=System Insight Agent
Documentation=https://github.com/Leptons1618/sia-proto
After=network.target

[Service]
Type=simple
User=sia
Group=sia
ExecStart=$INSTALL_DIR/sia-agent
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=sia-agent

# Runtime directory (creates /run/sia automatically)
RuntimeDirectory=sia
RuntimeDirectoryMode=0755

# Working directory
WorkingDirectory=$DATA_DIR

# Environment
Environment="RUST_LOG=info"
Environment="SIA_CONFIG=$CONFIG_DIR/config.toml"

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$DATA_DIR $SOCKET_DIR
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

# Resource limits
MemoryMax=200M
TasksMax=50

[Install]
WantedBy=multi-user.target
EOF

    # Set permissions
    chmod 644 "$SERVICE_DIR/sia-agent.service"
    chown root:root "$SERVICE_DIR/sia-agent.service"

    # Reload systemd
    echo "🔄 Reloading systemd daemon..."
    systemctl daemon-reload
else
    echo "⚠️  systemd not available (common in WSL). Skipping service installation."
    echo "   You can run sia-agent manually or set up your own service manager."
fi

echo ""
echo "✅ Installation complete!"
echo ""
if command -v systemctl &> /dev/null; then
    echo "📋 Next steps:"
    echo "   1. Start the service:     sudo systemctl start sia-agent"
    echo "   2. Enable on boot:        sudo systemctl enable sia-agent"
    echo "   3. Check status:          sia-cli status"
    echo "   4. View logs:             sudo journalctl -u sia-agent -f"
else
    echo "📋 Next steps (systemd not available):"
    echo "   1. Start manually:        sudo -u sia $INSTALL_DIR/sia-agent"
    echo "   2. Or run in background:  sudo -u sia $INSTALL_DIR/sia-agent &"
    echo "   3. Check status:          sia-cli status"
fi
echo ""
echo "📝 Configuration: $CONFIG_DIR/config.toml"
echo "💾 Database:      $DATA_DIR/sia.db"
echo "🔌 Socket:        $SOCKET_DIR/sia.sock"
echo ""
echo "🎯 You can now run 'sia-cli' from anywhere!"
