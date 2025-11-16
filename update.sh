#!/bin/bash
set -e

# SIA Update Script
# Rebuilds and updates installed binaries without removing config/data

INSTALL_DIR="/usr/local/bin"

echo "🔄 Updating SIA (System Insight Agent)..."

# Check if running as root/sudo
if [ "$EUID" -ne 0 ]; then
    echo "❌ Please run with sudo: sudo ./update.sh"
    exit 1
fi

# Get the original user (who ran sudo)
ORIGINAL_USER="${SUDO_USER:-$USER}"

# Get absolute path to project directory (works better in WSL)
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Check if service is running (may not be available in WSL)
SERVICE_WAS_RUNNING=false
if command -v systemctl &> /dev/null && systemctl is-active --quiet sia-agent 2>/dev/null; then
    echo "🛑 Stopping sia-agent service..."
    systemctl stop sia-agent
    SERVICE_WAS_RUNNING=true
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

# Install binaries
echo "🔧 Installing updated binaries to $INSTALL_DIR..."
cp target/release/sia-agent "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/sia-agent"

# Update TypeScript CLI
echo "🔧 Updating TypeScript CLI..."
cd "$PROJECT_DIR/cli-ts"
if [ -n "$SUDO_USER" ]; then
    sudo -u "$SUDO_USER" bash -c "cd '$PROJECT_DIR/cli-ts' && npm install && npm run build"
else
    npm install && npm run build
fi

# Update CLI files
mkdir -p /usr/lib/sia/cli-ts
cp -r "$PROJECT_DIR/cli-ts/dist" /usr/lib/sia/cli-ts/
cp "$PROJECT_DIR/cli-ts/package.json" /usr/lib/sia/cli-ts/
cd /usr/lib/sia/cli-ts
npm install --production --no-save

# Restart service if it was running (may not be available in WSL)
if [ "$SERVICE_WAS_RUNNING" = true ] && command -v systemctl &> /dev/null; then
    echo "▶️  Starting sia-agent service..."
    systemctl start sia-agent
    sleep 1
    if systemctl is-active --quiet sia-agent; then
        echo "✅ Service started successfully"
    else
        echo "⚠️  Service may have failed to start. Check logs: sudo journalctl -u sia-agent -n 20"
    fi
fi

echo ""
echo "✅ Update complete!"
echo ""
echo "📋 You can now:"
echo "   - Check status:  sia-cli status"
echo "   - View logs:     sudo journalctl -u sia-agent -f"
echo ""
